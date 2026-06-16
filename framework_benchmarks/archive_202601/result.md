================================================================================
Comprehensive Framework Comparison Benchmark
================================================================================
Started: 2026-02-01 14:05:31
Frameworks: MyQuat (Rust), Qiskit (Python), Cirq (Python)


============================================================
Problem: H2_4q: 4 qubits, 15 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.49 ms,    200 gates, depth    70
    Qiskit:       242.63 ms,    500 gates, depth   233
    Qiskit_Opt:    20.61 ms,    344 gates, depth   164
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         1.53 ms,    364 gates, depth   131
    Qiskit:       136.04 ms,    930 gates, depth   470
    Qiskit_Opt:    12.11 ms,    534 gates, depth   303
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         3.24 ms,   1000 gates, depth   350
    Qiskit:        14.67 ms,   2500 gates, depth  1153
    Qiskit_Opt:    21.52 ms,   1704 gates, depth   804
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         5.89 ms,   1804 gates, depth   651
    Qiskit:        22.83 ms,   4650 gates, depth  2350
    Qiskit_Opt:    41.70 ms,   2654 gates, depth  1503
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         9.93 ms,   3604 gates, depth  1301
    Qiskit:        48.61 ms,   9300 gates, depth  4700
    Qiskit_Opt:    70.29 ms,   5304 gates, depth  3003
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: LiH_6q: 6 qubits, 29 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.82 ms,    282 gates, depth    90
    Qiskit:         9.51 ms,   1040 gates, depth   733
    Qiskit_Opt:    12.51 ms,    764 gates, depth   514
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         2.41 ms,    436 gates, depth   141
    Qiskit:        12.30 ms,   2010 gates, depth  1470
    Qiskit_Opt:    19.78 ms,   1432 gates, depth  1025
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         5.80 ms,   1402 gates, depth   450
    Qiskit:        27.48 ms,   5200 gates, depth  3653
    Qiskit_Opt:    48.33 ms,   3804 gates, depth  2554
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        11.95 ms,   2156 gates, depth   701
    Qiskit:        52.64 ms,  10050 gates, depth  7350
    Qiskit_Opt:    85.02 ms,   7106 gates, depth  5103
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        22.45 ms,   4306 gates, depth  1401
    Qiskit:       111.96 ms,  20100 gates, depth 14700
    Qiskit_Opt:   179.51 ms,  14206 gates, depth 10203
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: Heisenberg_4q: 4 qubits, 12 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.16 ms,    320 gates, depth   100
    Qiskit:         4.66 ms,    680 gates, depth   520
    Qiskit_Opt:     8.67 ms,    364 gates, depth   241
    Cirq:         296.22 ms,    401 gates, depth   241

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         1.55 ms,    529 gates, depth   163
    Qiskit:         6.80 ms,   1330 gates, depth  1010
    Qiskit_Opt:     9.28 ms,    553 gates, depth   367
    Cirq:         272.92 ms,    401 gates, depth   241

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         3.51 ms,   1600 gates, depth   500
    Qiskit:        11.79 ms,   3400 gates, depth  2600
    Qiskit_Opt:   149.49 ms,   1804 gates, depth  1201
    Cirq:        1313.65 ms,   2001 gates, depth  1201

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         6.97 ms,   2609 gates, depth   803
    Qiskit:        20.32 ms,   6650 gates, depth  5050
    Qiskit_Opt:    34.80 ms,   2713 gates, depth  1807
    Cirq:        1301.52 ms,   2001 gates, depth  1201

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        11.18 ms,   5209 gates, depth  1603
    Qiskit:        40.70 ms,  13300 gates, depth 10100
    Qiskit_Opt:    58.60 ms,   5413 gates, depth  3607
    Cirq:        2824.89 ms,   3701 gates, depth  2401

============================================================
Problem: Heisenberg_6q: 6 qubits, 18 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.34 ms,    500 gates, depth   100
    Qiskit:         8.30 ms,   1020 gates, depth   780
    Qiskit_Opt:    11.92 ms,    546 gates, depth   361
    Cirq:         428.59 ms,    601 gates, depth   361

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         2.69 ms,    815 gates, depth   163
    Qiskit:         8.60 ms,   2010 gates, depth  1530
    Qiskit_Opt:    12.27 ms,    915 gates, depth   607
    Cirq:         416.41 ms,    601 gates, depth   361

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         5.71 ms,   2500 gates, depth   500
    Qiskit:        25.30 ms,   5100 gates, depth  3900
    Qiskit_Opt:    29.50 ms,   2706 gates, depth  1801
    Cirq:        1949.01 ms,   3001 gates, depth  1801

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         9.19 ms,   4015 gates, depth   803
    Qiskit:        28.29 ms,  10050 gates, depth  7650
    Qiskit_Opt:    44.48 ms,   4515 gates, depth  3007
    Cirq:        1950.53 ms,   3001 gates, depth  1801

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        18.82 ms,   8015 gates, depth  1603
    Qiskit:       161.80 ms,  20100 gates, depth 15300
    Qiskit_Opt:    92.95 ms,   9015 gates, depth  6007
    Cirq:        4093.39 ms,   5501 gates, depth  3601

============================================================
Problem: Heisenberg_8q: 8 qubits, 24 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         2.38 ms,    680 gates, depth   100
    Qiskit:         6.78 ms,   1360 gates, depth  1040
    Qiskit_Opt:    10.73 ms,    728 gates, depth   481
    Cirq:         559.01 ms,    801 gates, depth   481

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         3.22 ms,   1155 gates, depth   190
    Qiskit:        10.71 ms,   2690 gates, depth  2050
    Qiskit_Opt:    16.13 ms,   1277 gates, depth   847
    Cirq:         556.72 ms,    801 gates, depth   481

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         7.34 ms,   3400 gates, depth   500
    Qiskit:       121.71 ms,   6800 gates, depth  5200
    Qiskit_Opt:    33.84 ms,   3608 gates, depth  2401
    Cirq:        2566.08 ms,   4001 gates, depth  2401

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        15.07 ms,   5715 gates, depth   950
    Qiskit:        38.60 ms,  13450 gates, depth 10250
    Qiskit_Opt:    62.73 ms,   6317 gates, depth  4207
    Cirq:        2858.93 ms,   4001 gates, depth  2401

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        28.31 ms,  11415 gates, depth  1900
    Qiskit:        77.71 ms,  26900 gates, depth 20500
    Qiskit_Opt:   141.07 ms,  12617 gates, depth  8407
    Cirq:        6065.92 ms,   7301 gates, depth  4801

============================================================
Problem: TFIM_4q: 4 qubits, 7 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         0.75 ms,    123 gates, depth    40
    Qiskit:         3.65 ms,    130 gates, depth    73
    Qiskit_Opt:     5.32 ms,    130 gates, depth    73
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         0.77 ms,    126 gates, depth    41
    Qiskit:         3.77 ms,    250 gates, depth   200
    Qiskit_Opt:     5.76 ms,    184 gates, depth   143
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.34 ms,    603 gates, depth   200
    Qiskit:         6.16 ms,    650 gates, depth   353
    Qiskit_Opt:     9.63 ms,    650 gates, depth   353
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         2.20 ms,    606 gates, depth   201
    Qiskit:        10.15 ms,   1250 gates, depth  1000
    Qiskit_Opt:    14.64 ms,    904 gates, depth   703
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         3.88 ms,   1206 gates, depth   401
    Qiskit:        17.22 ms,   2500 gates, depth  2000
    Qiskit_Opt:    24.57 ms,   1804 gates, depth  1403
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: TFIM_6q: 6 qubits, 11 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         0.73 ms,    203 gates, depth    40
    Qiskit:         3.52 ms,    210 gates, depth    79
    Qiskit_Opt:     9.90 ms,    210 gates, depth    79
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         1.74 ms,    208 gates, depth    41
    Qiskit:         4.00 ms,    410 gates, depth   320
    Qiskit_Opt:     7.26 ms,    324 gates, depth   263
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.88 ms,   1003 gates, depth   200
    Qiskit:        11.01 ms,   1050 gates, depth   359
    Qiskit_Opt:    12.43 ms,   1050 gates, depth   359
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         3.58 ms,   1008 gates, depth   201
    Qiskit:        14.27 ms,   2050 gates, depth  1600
    Qiskit_Opt:    23.22 ms,   1604 gates, depth  1303
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         6.71 ms,   2008 gates, depth   401
    Qiskit:        27.15 ms,   4100 gates, depth  3200
    Qiskit_Opt:    41.16 ms,   3204 gates, depth  2603
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: TFIM_8q: 8 qubits, 15 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.18 ms,    283 gates, depth    40
    Qiskit:         3.79 ms,    290 gates, depth    85
    Qiskit_Opt:     7.25 ms,    290 gates, depth    85
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         1.50 ms,    290 gates, depth    41
    Qiskit:         5.31 ms,    570 gates, depth   440
    Qiskit_Opt:     9.00 ms,    464 gates, depth   383
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         2.58 ms,   1403 gates, depth   200
    Qiskit:        11.02 ms,   1450 gates, depth   365
    Qiskit_Opt:    15.77 ms,   1450 gates, depth   365
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         5.47 ms,   1410 gates, depth   201
    Qiskit:        22.72 ms,   2850 gates, depth  2200
    Qiskit_Opt:    27.73 ms,   2304 gates, depth  1903
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        10.71 ms,   2810 gates, depth   401
    Qiskit:        40.30 ms,   5700 gates, depth  4400
    Qiskit_Opt:    56.08 ms,   4604 gates, depth  3803
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: TFIM_10q: 10 qubits, 19 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.13 ms,    363 gates, depth    40
    Qiskit:         7.17 ms,    370 gates, depth    91
    Qiskit_Opt:     8.85 ms,    370 gates, depth    91
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         1.92 ms,    402 gates, depth    41
    Qiskit:         6.53 ms,    730 gates, depth   560
    Qiskit_Opt:    10.36 ms,    604 gates, depth   503
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         3.49 ms,   1803 gates, depth   200
    Qiskit:        16.87 ms,   1850 gates, depth   371
    Qiskit_Opt:    20.41 ms,   1850 gates, depth   371
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         8.55 ms,   1962 gates, depth   201
    Qiskit:        25.00 ms,   3650 gates, depth  2800
    Qiskit_Opt:    38.47 ms,   3004 gates, depth  2503
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        15.37 ms,   3912 gates, depth   401
    Qiskit:        50.31 ms,   7300 gates, depth  5600
    Qiskit_Opt:    69.28 ms,   6004 gates, depth  5003
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: Random_4q_20t: 4 qubits, 20 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:         1.89 ms,    590 gates, depth   200
    Qiskit:        15.57 ms,   2500 gates, depth  1620
    Qiskit_Opt:    23.78 ms,   1309 gates, depth   987
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:         3.92 ms,   1150 gates, depth   430
    Qiskit:        30.37 ms,   4870 gates, depth  3130
    Qiskit_Opt:    48.30 ms,   2581 gates, depth  1967
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:        10.32 ms,   2950 gates, depth  1000
    Qiskit:        67.63 ms,  12500 gates, depth  8100
    Qiskit_Opt:   114.82 ms,   6509 gates, depth  4907
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        31.07 ms,   5750 gates, depth  2150
    Qiskit:       248.58 ms,  24350 gates, depth 15650
    Qiskit_Opt:   219.91 ms,  12911 gates, depth  9807
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        96.78 ms,  11500 gates, depth  4300
    Qiskit:       248.92 ms,  48700 gates, depth 31300
    Qiskit_Opt:   428.68 ms,  25811 gates, depth 19607
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: Random_6q_30t: 6 qubits, 30 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:        27.78 ms,   1622 gates, depth   430
    Qiskit:        38.07 ms,   6280 gates, depth  3470
    Qiskit_Opt:    54.02 ms,   3470 gates, depth  2640
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:        98.01 ms,   3122 gates, depth   830
    Qiskit:        63.99 ms,  12410 gates, depth  6830
    Qiskit_Opt:   109.05 ms,   6655 gates, depth  5089
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:       654.91 ms,   8102 gates, depth  2150
    Qiskit:       150.58 ms,  31400 gates, depth 17350
    Qiskit_Opt:   257.62 ms,  17350 gates, depth 13200
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:      2463.74 ms,  15602 gates, depth  4150
    Qiskit:       296.84 ms,  62050 gates, depth 34150
    Qiskit_Opt:   588.01 ms,  33215 gates, depth 25409
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
    MyQuat:     12446.52 ms,  31202 gates, depth  8300
    Qiskit:       577.23 ms, 124100 gates, depth 68300
    Qiskit_Opt:  1016.72 ms,  66415 gates, depth 50809
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

============================================================
Problem: Random_8q_40t: 8 qubits, 40 terms
============================================================

  Config: steps=10, time=1.0, order=1
  --------------------------------------------------
    MyQuat:       217.29 ms,   3222 gates, depth  1392
    Qiskit:        53.32 ms,  10880 gates, depth  5770
    Qiskit_Opt:    88.96 ms,   6096 gates, depth  4643
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=10, time=1.0, order=2
  --------------------------------------------------
    MyQuat:       829.71 ms,   6493 gates, depth  2772
    Qiskit:       100.86 ms,  21410 gates, depth 11330
    Qiskit_Opt:   179.56 ms,  11944 gates, depth  9089
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=1
  --------------------------------------------------
    MyQuat:      6249.77 ms,  16102 gates, depth  6952
    Qiskit:       269.17 ms,  54400 gates, depth 28850
    Qiskit_Opt:   585.62 ms,  30456 gates, depth 23203
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=50, time=1.0, order=2
  --------------------------------------------------
    MyQuat:     28674.41 ms,  32453 gates, depth 13852
    Qiskit:       536.52 ms, 107050 gates, depth 56650
    Qiskit_Opt:   989.57 ms,  59664 gates, depth 45409
    Cirq:       FAILED - Given DensePauliString doesn't have +1 a

  Config: steps=100, time=1.0, order=2
  --------------------------------------------------
Terminated