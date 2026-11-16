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
    % Export the named MATPOWER 8.1 dual variables in internal model order,
    % but bind every row back to the original external source identity.
    % These are solver-reported observations; the Rust KKT verifier recomputes
    % every sign, constraint, stationarity and complementarity condition.
    if ~isfield(results, 'lin') || ~isfield(results, 'var') || ...
            ~isfield(results.lin, 'mu') || ~isfield(results.var, 'mu')
        error('MATPOWER result is missing named linear-constraint or variable multipliers');
    end
    if ~isfield(results.lin.mu, 'l') || ~isfield(results.lin.mu, 'u') || ...
            ~isfield(results.var.mu, 'l') || ~isfield(results.var.mu, 'u')
        error('MATPOWER result is missing lower/upper multiplier groups');
    end
    if ~isfield(results.lin.mu.l, 'Pmis') || ~isfield(results.lin.mu.u, 'Pmis') || ...
            ~isfield(results.var.mu.l, 'Va') || ~isfield(results.var.mu.u, 'Va') || ...
            ~isfield(results.var.mu.l, 'Pg') || ~isfield(results.var.mu.u, 'Pg')
        error('MATPOWER result is missing required Pmis, Va or Pg multiplier blocks');
    end

    fprintf(fid, 'DUAL_META\t1\tlower_upper_nonnegative\n');
    bus_ids = results.order.bus.i2e(:);
    va_mu_l = results.var.mu.l.Va(:);
    va_mu_u = results.var.mu.u.Va(:);
    pg_mu_l = results.var.mu.l.Pg(:);
    pg_mu_u = results.var.mu.u.Pg(:);
    pmis_mu_l = results.lin.mu.l.Pmis(:);
    pmis_mu_u = results.lin.mu.u.Pmis(:);
    if length(bus_ids) ~= length(va_mu_l) || length(bus_ids) ~= length(va_mu_u) || ...
            length(bus_ids) ~= length(pmis_mu_l) || length(bus_ids) ~= length(pmis_mu_u)
        error('MATPOWER internal bus/dual dimensions do not agree');
    end
    if length(results.order.gen.i2e) ~= length(pg_mu_l) || ...
            length(results.order.gen.i2e) ~= length(pg_mu_u)
        error('MATPOWER internal generator/dual dimensions do not agree');
    end

    for i = 1:length(bus_ids)
        fprintf(fid, 'DUAL_BUS\t%d\t%.17g\t%.17g\n', ...
            round(bus_ids(i)), pmis_mu_l(i), pmis_mu_u(i));
        fprintf(fid, 'DUAL_VA\t%d\t%.17g\t%.17g\n', ...
            round(bus_ids(i)), va_mu_l(i), va_mu_u(i));
    end
    for i = 1:length(results.order.gen.i2e)
        fprintf(fid, 'DUAL_GEN\t%d\t%.17g\t%.17g\n', ...
            round(results.order.gen.i2e(i)), pg_mu_l(i), pg_mu_u(i));
    end

    % Use MATPOWER's explicit internal-to-external branch mapping rather
    % than depending on the optional legacy OPF model object being retained.
    branch_source_rows_all = results.order.branch.i2e(:);
    source_rate_a = mpc.branch(branch_source_rows_all, RATE_A);
    constrained_internal = find(source_rate_a ~= 0 & source_rate_a < 1e10);
    if ~isempty(constrained_internal)
        if ~isfield(results.lin.mu.l, 'Pf') || ~isfield(results.lin.mu.u, 'Pf')
            error('MATPOWER result is missing the Pf dual block for constrained branches');
        end
        pf_mu_l = results.lin.mu.l.Pf(:);
        pf_mu_u = results.lin.mu.u.Pf(:);
        if length(constrained_internal) ~= length(pf_mu_l) || ...
                length(constrained_internal) ~= length(pf_mu_u)
            error('MATPOWER Pf dual dimensions do not match constrained branches');
        end
        branch_source_rows = results.order.branch.i2e(constrained_internal);
        for i = 1:length(constrained_internal)
            fprintf(fid, 'DUAL_FLOW\t%d\t%.17g\t%.17g\n', ...
                round(branch_source_rows(i)), pf_mu_l(i), pf_mu_u(i));
        end
    end

    source_angle_min = mpc.branch(branch_source_rows_all, ANGMIN);
    source_angle_max = mpc.branch(branch_source_rows_all, ANGMAX);
    angle_mask = (source_angle_min ~= 0 & source_angle_min > -360) | ...
        (source_angle_max ~= 0 & source_angle_max < 360) | ...
        (source_angle_min ~= 0 & source_angle_max == 0) | ...
        (source_angle_min == 0 & source_angle_max ~= 0);
    angle_internal = find(angle_mask);
    if ~isempty(angle_internal)
        if ~isfield(results.lin.mu.l, 'ang') || ~isfield(results.lin.mu.u, 'ang')
            error('MATPOWER result is missing the angle-difference dual block');
        end
        ang_mu_l = results.lin.mu.l.ang(:);
        ang_mu_u = results.lin.mu.u.ang(:);
        if length(angle_internal) ~= length(ang_mu_l) || ...
                length(angle_internal) ~= length(ang_mu_u)
            error('MATPOWER angle dual dimensions do not match constrained branches');
        end
        angle_source_rows = results.order.branch.i2e(angle_internal);
        for i = 1:length(angle_internal)
            fprintf(fid, 'DUAL_ANGLE\t%d\t%.17g\t%.17g\n', ...
                round(angle_source_rows(i)), ang_mu_l(i), ang_mu_u(i));
        end
    end
end
