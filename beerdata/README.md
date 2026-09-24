# beerdata

A single-binary Rust CLI that collects country-level beer affordability data and creates quarterly datapoints in Craft CMS.

## Sources and units

- **Numbeo via Apify:** the country-level _Domestic Beer (0.5 Liter Bottle)_ average under Markets (grocery-store price), and _Average Monthly Net Salary (After Tax)_ under Salaries And Financing. Both come from the same cost-of-living scrape. Beer prices outside their reported low/high range are rejected; salaries must be positive and in the beer price's reported currency. Countries without a usable salary are not uploaded. The Apify actor uses residential proxies and new runs may incur charges.
- **USD exchange rates:** [fawazahmed0/currency-api](https://github.com/fawazahmed0/exchange-api), a no-key, daily-updated feed with a Cloudflare fallback. One USD-to-local-currency rate snapshot is saved per quarter in `tmp/fx-<quarter-id>.json` and reused on reruns. It is the latest available rate when first fetched, **not necessarily the rate on Numbeo's scrape date**. Preserve it alongside your Craft backup for reproducibility.

An estimated hourly **net** wage assumes 40 hours per week, or `40 × 52 / 12 = 173⅓` hours per month. If `r` is the rate in units of the reported currency per USD:

```text
localWage = Numbeo net monthly salary / (40 × 52 / 12)
price     = localPrice / r                    (USD per bottle)
wage      = localWage / r                     (USD per estimated hour)
minutes   = price / wage × 60
```

Craft stores `localCurrency` (Numbeo's currency code), `localPrice` (per bottle), `localWage` (estimated net per hour), `price` (USD per bottle), and `wage` (estimated net USD per hour). The frontend computes minutes from the USD fields. Because price and wage use the **same** rate, the affordability ratio is independent of the conversion; USD is for comparing/displaying dollar amounts. The hourly figure is an estimate, not a directly observed hourly wage. Craft's `price` and `wage` fields store two decimal places, so the displayed ratio is approximate.

Numbeo's figures are crowdsourced and may contain outliers; neither salary nor price is guaranteed to represent a typical individual. If a required currency rate is unavailable, the sync fails before creating datapoints rather than saving mixed units. The snapshot used for the Sep 2026 data had 179 usable beer-priced countries, 178 with a matching salary; coverage may change.

## Commands

Run from the `beerdata` directory:

```sh
cargo run -- countries
cargo run -- push-countries --endpoint-url URL --endpoint-token TOKEN
cargo run -- sync --endpoint-url URL --endpoint-token TOKEN --apify-api-token TOKEN --quarter-id 4 --cached-only
```

`countries` writes `countries.json`. `push-countries` is for the initial, one-time creation of Craft country entries; it does not check for existing countries.

`sync` checks Craft countries and existing datapoints, loads or fetches the quarter's USD rates, reads the country list, and uses Apify to obtain Numbeo rows. **Use `--cached-only` to re-import the existing Apify cache without starting another paid actor run.** Omit it only when you intend to request uncached countries. An existing active Apify run is resumed; `--resume-apify-run-id ID` can import an older run not already in the cache. The quarter ID is a Craft relation and cache key; it is not sent to Numbeo or the exchange-rate feed.

The raw actor rows and run ID are kept in `tmp/apify-<quarter-id>.json`; extracted country prices, currencies, and monthly salaries are written to `numbeo.json`. `tmp/` and `numbeo.json` are git-ignored. Keep the Apify and FX cache files with your backups. If the Apify run aborts, its retrieved rows are saved to the cache, and `--cached-only` can ingest the available data without another paid run.

The Craft upload skips an existing `(quarter, country)` datapoint and duplicate cached prices, so it can resume after a partial upload. **It does not update old datapoints:** if a quarter still has entries created under the previous local-currency/ILOSTAT pipeline, remove or migrate those entries before syncing USD values. Missing Numbeo salaries are skipped rather than written as zero.

Tokens passed on the command line can appear in shell history and process listings; rotate any previously exposed tokens.
