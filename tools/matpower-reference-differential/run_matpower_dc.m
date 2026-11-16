function run_matpower_dc(case_path, output_path, matpower_root, mp_opt_model_root, mips_root, mp_test_root, source_git_blob_sha1)
    % Run MATPOWER 8.1's own DC power flow on unchanged source bytes.
    % MATPOWER, MP-Opt-Model and MP-Test are separate pinned source packages.
    %
    % Usage:
    %   run_matpower_dc(case_path, output_path, matpower_root, ...
    %       mp_opt_model_root, mips_root, mp_test_root, source_git_blob_sha1)

    if nargin ~= 7
        error('Expected case, output, MATPOWER root, MP-Opt-Model root, MIPS root, MP-Test root, digest');
    end
    if length(source_git_blob_sha1) ~= 40 || ...
            isempty(regexp(source_git_blob_sha1, '^[0-9a-fA-F]{40}$', 'once'))
        error('source_git_blob_sha1 must be exactly 40 hexadecimal digits');
    end

    % Resolve every path to an absolute path before loadcase(). MATPOWER's
    % feval_w_path temporarily changes the working directory while evaluating
    % case functions; Octave discards relative search paths during that change.
    case_path = absolute_existing_file(case_path, 'case file');
    matpower_root = absolute_existing_directory(matpower_root, 'MATPOWER root');
    mp_opt_model_root = absolute_existing_directory(mp_opt_model_root, 'MP-Opt-Model root');
    mp_test_root = absolute_existing_directory(mp_test_root, 'MP-Test root');
    output_path = absolute_output_path(output_path);

    matpower_lib = fullfile(matpower_root, 'lib');
    mp_opt_model_lib = fullfile(mp_opt_model_root, 'lib');
    mips_lib = fullfile(mips_root, 'lib');
    mp_test_lib = fullfile(mp_test_root, 'lib');
    for required = {matpower_lib, mp_opt_model_lib, mips_lib, mp_test_lib}
        if exist(required{1}, 'dir') ~= 7
            error('Required reference library directory is absent: %s', required{1});
        end
    end

    addpath(genpath(matpower_lib));
    addpath(genpath(mp_opt_model_lib));
    addpath(genpath(mips_lib));
    addpath(genpath(mp_test_lib));

    mpc = loadcase(case_path);

    % loadcase/feval_w_path temporarily adjusts the Octave search path while
    % evaluating the case function. Re-admit only the three actual library
    % roots after it returns; do not recursively add docs/examples.
    addpath(genpath(matpower_lib));
    addpath(genpath(mp_opt_model_lib));
    addpath(genpath(mips_lib));
    addpath(genpath(mp_test_lib));
    define_constants;

    mpopt = mpoption('verbose', 0, 'out.all', 0);
    results = rundcpf(mpc, mpopt);
    if ~isfield(results, 'success') || ~results.success
        error('MATPOWER rundcpf did not converge for %s', case_path);
    end

    ref_rows = find(round(results.bus(:, BUS_TYPE)) == REF);
    if numel(ref_rows) ~= 1
        error('The reference panel requires exactly one reference bus; found %d', numel(ref_rows));
    end
    ref_row = ref_rows(1);
    reference_bus = round(results.bus(ref_row, BUS_I));
    reference_angle_degrees = results.bus(ref_row, VA);

    fid = fopen(output_path, 'w');
    if fid < 0
        error('Cannot open output TSV: %s', output_path);
    end
    cleanup = onCleanup(@() fclose(fid));

    fprintf(fid, 'META\tsource_git_blob_sha1\t%s\n', lower(source_git_blob_sha1));
    fprintf(fid, 'META\tmatpower_version\t8.1\n');
    fprintf(fid, 'META\tmp_opt_model_version\t5.0\n');
    fprintf(fid, 'META\tmips_version\t1.5.2\n');
    fprintf(fid, 'META\tmp_test_version\t8.1\n');
    fprintf(fid, 'META\tbase_mva\t%.17g\n', results.baseMVA);
    fprintf(fid, 'META\treference_bus\t%d\n', reference_bus);

    for i = 1:size(results.bus, 1)
        row = results.bus(i, :);
        bus_id = round(row(BUS_I));
        generator_rows = results.gen( ...
            round(results.gen(:, GEN_BUS)) == bus_id & results.gen(:, GEN_STATUS) > 0, :);
        online_generation_mw = sum(generator_rows(:, PG));
        net_injection_mw = online_generation_mw - row(PD);
        angle_relative_to_reference_degrees = row(VA) - reference_angle_degrees;
        fprintf(fid, 'BUS\t%d\t%.17g\t%.17g\n', ...
            bus_id, angle_relative_to_reference_degrees, net_injection_mw);
    end

    for i = 1:size(results.branch, 1)
        row = results.branch(i, :);
        fprintf(fid, 'BRANCH\t%d\t%d\t%d\t%d\t%.17g\t%.17g\n', ...
            i, round(row(F_BUS)), round(row(T_BUS)), ...
            round(row(BR_STATUS) ~= 0), row(PF), row(PT));
    end
end

function resolved = absolute_existing_file(path, label)
    [ok, metadata] = fileattrib(path);
    if ~ok || exist(path, 'file') ~= 2
        error('%s does not exist or is not accessible: %s', label, path);
    end
    resolved = metadata.Name;
end

function resolved = absolute_existing_directory(path, label)
    [ok, metadata] = fileattrib(path);
    if ~ok || exist(path, 'dir') ~= 7
        error('%s does not exist or is not accessible: %s', label, path);
    end
    resolved = metadata.Name;
end

function resolved = absolute_output_path(path)
    [parent, filename, extension] = fileparts(path);
    if isempty(parent)
        parent = pwd;
    end
    parent = absolute_existing_directory(parent, 'output directory');
    resolved = fullfile(parent, [filename extension]);
end
