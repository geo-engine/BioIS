# Climate risk: occurrence probability

The climate-risk tables rate each indicator (heat days, frost days, …) with an occurrence
probability and the FMEA/ISO-inspired probability classes 1–10. This page explains which event
the probability refers to and why.

## The event: the reference period's 10-year event

For each indicator, the event is **a year with more event days than the 1-in-10-year level of the
reference period** (by default 1981–2010). Example: in 1981–2010, Berlin had more than 11 heat days
(≥ 30 °C) in about one year out of ten. The occurrence probability states how likely such a year is
in the analysis period.

This follows the IPCC AR6 Working Group I approach to extremes. The IPCC reports changes in the
frequency of **10-year events**: events that occurred on average once per decade in a reference
climate. It also reports how many times more often they occur under warming. See
[Figure SPM.6](https://www.ipcc.ch/report/ar6/wg1/figures/summary-for-policymakers/figure-spm-6/)
in the [Summary for Policymakers](https://www.ipcc.ch/report/ar6/wg1/chapter/summary-for-policymakers/)
and [Chapter 11, "Weather and Climate Extreme Events in a Changing Climate"](https://www.ipcc.ch/report/ar6/wg1/chapter/chapter-11/).

We apply the same idea to the yearly counts of event days, i.e. "a year with unusually many heat
days". IPCC SPM.6 applies it to daily values ("a day hotter than the 10-year level").

## Computation

Per climate model:

1. **Level.** The 10-year level is the nearest-rank 90th percentile of the model's yearly counts in
   the reference period. Without ties, 3 of 30 reference years lie above it. Each model gets its
   own level, so a model's bias (e.g. running warm) does not count as a change.
2. **Exceedances.** We count the years above the model's level, in the analysis period and in the
   reference period.

Over all models that have both periods:

- **Occurrence probability** = years above the level ÷ all years, in the analysis period. This value
  is mapped to the probability classes.
- **Risk ratio** = occurrence probability in the analysis period ÷ the same in the reference period.
  This is the IPCC "probability ratio": 3.0 means the former 10-year event now happens three times
  as often. It is empty if the event never occurred in the reference period.

The code is in `backend/src/processes/climate_risk/compute.rs`, in `ten_year_level`,
`ten_year_event_counts`, `wilson_interval` and `risk_ratio`.

## Reference period: always 30 years

The analysis period can be 5 to 30 years long. The reference period is always 30 years (a WMO
climate normal, by default 1981–2010), independent of the analysis period. A 10-year level needs
enough reference years. With a 5-year reference, the level would simply be the reference maximum,
and the event would never occur in the reference period, so it would have no risk ratio. With a
fixed 30-year reference, every analysis period, short or long, is compared with the same event.

## Uncertainty: class range in the label

The probability is estimated from few model-years when the analysis period is short (5 years ×
5 models = 25). The label therefore uses the 95 % [Wilson score interval](https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval#Wilson_score_interval)
of the probability:

- **Interval inside one class:** the label shows that class, e.g. "10 · extreme (40 %)".
- **Interval crosses class borders:** the label shows the class range, e.g.
  "7–10 · high to extreme (12 %)". The percentage is always the best estimate. The color is the
  color of its class.

Examples with 5 models:

| Analysis period | Model-years | Probability | 95 % interval | Label |
|---|---|---|---|---|
| 5 years | 25 | 12 % | 4–30 % | 7–10 · high to extreme (12 %) |
| 5 years | 25 | 40 % | 23–59 % | 10 · extreme (40 %) |
| 30 years | 150 | 12 % | 8–18 % | 8–9 · very high to extremely high (12 %) |

Model-years count as independent samples. The models are independent, and the year-to-year
correlation of annual counts is small.

## Reading the classes

- **Reference period.** The probability is 0.1 by construction (3 of 30 years per model), which is
  class 9 (1-in-10 years). Ties in the counts can push it lower.
- **Analysis period.** Class 10 (p ≥ 0.2) means the former 10-year event occurs at least every five
  years, i.e. a risk ratio of about 2 or more. Strong warming signals (heat, tropical nights) reach
  this already in the near future. That is a real result, not a saturation artifact.
- **Decreasing hazards.** For frost days the probability falls below 0.1, which gives lower classes.
- **Rare indicators.** For heavy rain days the 10-year level is often 0 or 1 day, so the event is
  close to "at least one (or two) heavy rain days in the year".

The previous definition, "at least one event day in the year", is near-certain for most indicators.
It put almost every value into class 10.

## Limitations

- **Resolution.** Empirical quantiles from 30 reference years resolve events down to about
  1-in-10 to 1-in-20 years. The classes for rarer events (1 to 8, return periods of 50 years and
  more) would need an extreme-value fit (e.g. GEV) of the reference years. They are not used today.
- **Reference period.** The reference period is 1981–2010 from the historical simulations, not
  IPCC's pre-industrial 1850–1900. The risk ratio therefore measures change relative to the recent
  past.
