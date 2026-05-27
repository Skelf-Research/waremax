# WareMax-RL results (mean +/- 95% CI)


## standard_tight

```
policy                       SLA on-time %    p95 lateness s    throughput/h
----------------------------------------------------------------------------
nearest_robot (n=8)              97.9+-2.4         7.5+-11.6      67.0+-13.0
least_busy (n=8)                81.5+-11.5        47.9+-28.5      65.5+-11.7
round_robin (n=8)                98.2+-2.8          1.4+-2.6      66.0+-14.0
auction (n=8)                    97.9+-2.4         7.5+-11.6      67.0+-13.0
ppo/attribution/candidate (n=3)       96.4+-5.5         10.1+-5.9       66.0+-1.2
ppo/attribution/mlp (n=3)        82.4+-3.8        40.3+-10.0       65.3+-0.7
ppo/dense/candidate (n=3)       84.6+-12.9        35.6+-28.2       65.0+-1.2
ppo/dense/mlp (n=3)              82.5+-5.5         43.9+-2.7       64.2+-5.7
ppo/routed/candidate (n=3)       96.9+-4.2          8.4+-2.6       66.2+-0.7
ppo/routed/mlp (n=3)            84.6+-11.2        37.4+-27.4       65.5+-1.2
ppo/sparse/candidate (n=3)      88.0+-24.1        27.0+-48.9       65.3+-3.6
```

## standard_mod

```
policy                       SLA on-time %    p95 lateness s    throughput/h
----------------------------------------------------------------------------
nearest_robot (n=8)             100.0+-0.0          0.0+-0.0      67.0+-13.0
least_busy (n=8)                 97.5+-3.1         9.3+-11.1      65.5+-11.7
round_robin (n=8)               100.0+-0.0          0.0+-0.0      66.0+-14.0
auction (n=8)                   100.0+-0.0          0.0+-0.0      67.0+-13.0
ppo/attribution/candidate (n=3)      100.0+-0.0          0.0+-0.0       66.7+-1.4
ppo/dense/candidate (n=3)        99.4+-2.7          2.1+-9.1       65.8+-1.9
ppo/routed/candidate (n=3)       98.2+-4.2         6.1+-14.6       64.2+-6.8
ppo/sparse/candidate (n=3)       99.8+-0.8          0.6+-2.5       65.8+-0.7
```

## peak_tight

```
policy                       SLA on-time %    p95 lateness s    throughput/h
----------------------------------------------------------------------------
nearest_robot (n=8)              97.9+-3.0         7.4+-10.7     201.2+-20.2
least_busy (n=8)                 94.5+-5.7        17.3+-15.6     195.6+-15.9
round_robin (n=8)               100.0+-0.0          0.0+-0.0     208.8+-20.6
auction (n=8)                    97.9+-3.0         7.4+-10.7     201.2+-20.2
ppo/attribution/candidate (n=3)       95.6+-7.7        14.5+-20.0      196.9+-8.2
ppo/dense/candidate (n=3)        99.1+-2.7          3.0+-6.8      201.7+-7.0
ppo/routed/candidate (n=3)       94.7+-5.9        16.2+-11.6      199.4+-8.6
ppo/sparse/candidate (n=3)       94.7+-4.6        16.1+-16.2      199.0+-9.1
```

## congested

```
policy                       SLA on-time %    p95 lateness s    throughput/h
----------------------------------------------------------------------------
nearest_robot (n=8)             59.8+-15.8       230.3+-78.8     135.0+-31.5
least_busy (n=8)                 40.8+-8.8       334.2+-57.8     113.0+-14.3
round_robin (n=8)               66.5+-20.4      211.2+-130.7     143.0+-37.1
auction (n=8)                   59.8+-15.8       230.3+-78.8     135.0+-31.5
ppo/attribution/candidate (n=8)       56.0+-5.6       266.1+-29.5      129.4+-7.7
ppo/attribution_full/candidate (n=8)       51.6+-4.5       282.3+-22.5      120.8+-5.4
ppo/dense/candidate (n=3)       48.3+-23.7       307.9+-98.9     119.2+-22.1
ppo/routed/candidate (n=3)      58.5+-11.4       233.4+-58.9     139.7+-26.2
```
