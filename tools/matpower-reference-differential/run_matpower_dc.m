function run_matpower_dc(case_path, output_path, matpower_root, source_git_blob_sha1)
    % Run MATPOWER 8.1's own DC power flow on the unchanged source case.
    % This reference path is independent of the Sol-Atlas parser and solver.
    %
    % Usage:
    %   run_matpower_dc(case_path, output_path, matpower_root, source_git_blob_sha1)

    if nargin ~= 4
        error('Expected case_path, output_path, matpower_root, source_git_blob_sha1');
    end
    if length(source_git_blob_sha1) ~= 40 || ...
            isempty(regexp(source_git_blob_sha1, '^[0-9a-fA-F]{40}$', 'once'))
        error('source_git_blob_sha1 must be exactly 40 hexadecimal digits');
    end

    matpower_lib = fullfile(matpower_root, 'lib');
    addpath(genpath(matpower_lib));

    mpc = loadcase(case_path);

    % loadcase/feval_w_path temporarily adjusts Octave's path while evaluating
    % a case function. Re-admit only MATPOWER's actual function library after it
    % returns; do not recursively add docs, examples or optional submodule trees.
    addpath(genpath(matpower_lib));
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
