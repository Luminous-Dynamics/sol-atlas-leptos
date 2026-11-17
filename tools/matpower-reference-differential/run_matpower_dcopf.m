function run_matpower_dcopf(case_path, output_path, matpower_root, mp_opt_model_root, mips_root, mp_test_root, source_git_blob_sha1)
    % Solve one MATPOWER DC-OPF case with the immutable MATPOWER 8.1 stack.
    % The receipt is an observation from rundcopf, not a self-authenticating proof.
    %
    % The profile ID deliberately names the MATPOWER makeBdc convention:
    % x-only branch susceptance with transformer taps and phase shifts.

    if nargin ~= 7
        error('Expected case, output, MATPOWER, MP-Opt-Model, MIPS, MP-Test and source digest');
    end
    if length(source_git_blob_sha1) ~= 40 || ...
            isempty(regexp(source_git_blob_sha1, '^[0-9a-fA-F]{40}$', 'once'))
        error('source_git_blob_sha1 must be exactly 40 hexadecimal digits');
    end

    addpath(genpath(matpower_root));
    addpath(genpath(mp_opt_model_root));
    addpath(genpath(mips_root));
    addpath(genpath(mp_test_root));
    define_constants;

    mpc = loadcase(case_path);
    mpopt = mpoption('model', 'DC', 'verbose', 0, 'out.all', 0);
    results = rundcopf(mpc, mpopt);
    if ~isfield(results, 'success') || ~results.success
        error('MATPOWER rundcopf did not converge for %s', case_path);
    end

    fid = fopen(output_path, 'w');
    if fid < 0
        error('Cannot open DC-OPF output TSV: %s', output_path);
    end
    cleanup = onCleanup(@() fclose(fid));

    fprintf(fid, 'META\tsource_git_blob_sha1\t%s\n', lower(source_git_blob_sha1));
    fprintf(fid, 'META\tmatpower_version\t8.1\n');
    fprintf(fid, 'META\tmodel_profile_id\tmatpower_8_1_makeBdc_dcopf\n');
    fprintf(fid, 'META\tbase_mva\t%.17g\n', results.baseMVA);
    fprintf(fid, 'META\tobjective\t%.17g\n', results.f);

    for i = 1:size(results.bus, 1)
        row = results.bus(i, :);
        fprintf(fid, 'BUS\t%d\t%.17g\n', ...
            round(row(BUS_I)), row(VA));
    end

    for i = 1:size(results.gen, 1)
        row = results.gen(i, :);
        fprintf(fid, 'GEN\t%d\t%d\t%d\t%.17g\n', ...
            i, round(row(GEN_BUS)), round(row(GEN_STATUS) ~= 0), row(PG));
    end

    for i = 1:size(results.branch, 1)
        row = results.branch(i, :);
        fprintf(fid, 'BRANCH\t%d\t%d\t%d\t%d\t%.17g\t%.17g\n', ...
            i, round(row(F_BUS)), round(row(T_BUS)), ...
            round(row(BR_STATUS) ~= 0), row(PF), row(PT));
    end
end
