# Launch-only contract evidence

Source commit `77390b6` adds request-client tests that observe the actual endpoint:
observed-busy and became-busy-after-observation both attempt `/launch`, propagate
refusal and never try `/resume`. Legacy start chooses `/resume` for a busy host.

CI run [38421896732](https://github.com/cederikdotcom/moonlight-common-rust/actions/runs/38421896732)
passed both new tests and compared the full suite against pinned baseline
`a04b3085`. Both have the same four failures:

- `http::pair::client::test::pair_already_in_progress`
- `http::test::request_box_art`
- `http::test::request_client_info`
- `http::test::request_launch_and_resume`

This is a passing regression comparison, not an entirely passing library suite.
The workflow requires completed test-suite events and rejects additional failures;
it retains JSON results as an artifact. Native GLib/GStreamer dev dependencies
are installed on the CI runner. The existing unused pairing fixture is allowed
as a warning. No live provider, body or Samsung unit was exercised.

The initial PR merge against newer master failed because ClientInfo changed;
the PR now targets the consumer's pinned maintenance base. Recheck its current
merge CI before merging. Forward-porting to newer master is separate work.
