# Managed launch compatibility patch (#770)

This patch targets `maintenance/signage-a04b3085`, the exact library revision
used by hydra-moonlight-web-stream before #770. Current master has changed the
ClientInfo API, so this is a maintenance patch for that consumer rather than a
completed forward-port to master. The consumer pins source commit `77390b6`.

Async `MoonlightHost::launch_stream` sends `/launch` even when server information
reports a current game, and propagates launch refusal without trying `/resume`.
This avoids the race in checking `current_game` before calling the legacy
`start_stream`, which rechecks host state and resumes any running game. Existing
`start_stream` retains its previous endpoint selection.

The managed consumer must still enforce its assigned app/body and profile,
control concurrent admission and confirm owned-resource cleanup. This API does
not itself prove Sunshine ownership or target-device operation. No release or
production deployment is included in this draft.
