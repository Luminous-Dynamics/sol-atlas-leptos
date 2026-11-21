function mpc = two_bus_binding_line_dc_opf
mpc.version = '2';
mpc.baseMVA = 100;
mpc.bus = [
    1 3   0   0 0 0 1 1  0.000000000 230 1 1.1 0.9;
    2 1 100   0 0 0 1 1 -2.291831181 230 1 1.1 0.9;
];
mpc.gen = [
    1 40 0 100 -100 1 100 1 200 0;
    2 60 0 100 -100 1 100 1 200 0;
];
mpc.branch = [
    1 2 0.01 0.1 0 40 40 40 0 0 1 -30 30;
];
mpc.gencost = [
    2 0 0 2 10 0;
    2 0 0 2 30 0;
];
