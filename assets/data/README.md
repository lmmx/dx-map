## Data sources

Data are pre-prepared in Polars from `tubeulator` and served via FastAPI (see the `src` subdirectory
of the lmmx/tb8 repo, in particular `src/utils/api_functions` [here](https://github.com/lmmx/tb8/blob/master/app/api-explorer/src/utils/api_functions.js)).
These are served via Render but it is fine to just store them and package into this Rust app to
avoid having to wait for the Render server to spin up. We can assume the stations and platforms will
be relatively stable over time.

- `stations.json` comes from GET to `https://tb8.onrender.com/stations?query=SELECT%20DISTINCT%20ON%20(StationUniqueId)%20*%20FROM%20self%3B`
- `platforms.json` comes from GET to `https://tb8.onrender.com/platforms?query=SELECT%20*%20FROM%20self%3B`
- `routes.json` comes from combining the routes obtained by individual calls to `https://tb8.onrender.com/route-sequence-by-line-direction?line=bakerloo&direction=inbound`
  using the `combine_routes.sh` script which passes the files to jq and puts them in a single object under `.$line.$direction` for the following lines:
  - bakerloo
  - central
  - circle
  - district
  - hammersmith-city
  - jubilee
  - metropolitan
  - northern
  - piccadilly
  - victoria
  - waterloo-city
