function run_matpower_dc(case_path, output_path, matpower_root)
    % Run MATPOWER's independent DC power flow and write a stable TSV receipt.
    % This script does not use the Sol-Atlas network projection or solver.

    addpath(genpath(matpower_root));
    define_constants;

    mpc = loadcase(case_path);
    mpopt = mpoption('verbose', 0, 'out.all', 0);
    results = rundcpf(mpc, mpopt);
    if ~isfield(results, 'success') || ~results.success
        error('MATPOWER rundcpf did not converge for %s', case_path);
    end

    fid = fopen(output_path, 'w');
    if fid < 0
        error('Cannot open output TSV: %s', output_path);
    end
    cleanup = onCleanup(@() fclose(fid));

    fprintf(fid, 'META\tbase_mva\t%.17g\n', results.baseMVA);

    for i = 1:size(results.bus, 1)
        row = results.bus(i, :);
        fprintf(fid, 'BUS\t%d\t%.17g\t%.17g\t%.17g\t%d\n', ...
            round(row(BUS_I)), row(VA), row(PD), row(QD), round(row(BUS_TYPE)));
    end

    for i = 1:size(results.gen, 1)
        row = results.gen(i, :);
        fprintf(fid, 'GEN\t%d\t%d\t%d\t%.17g\n', ...
            i, round(row(GEN_BUS)), round(row(GEN_STATUS) ~= 0), row(PG));
    end

    for i = 1:size(results.branch, 1)
        row = results.branch(i, :);
        fprintf(fid, 'BRANCH\t%d\t%d\t%d\t%d\t%.17g\n', ...
            i, round(row(F_BUS)), round(row(T_BUS)), ...
            round(row(BR_STATUS) ~= 0), row(PF));
    end
end
