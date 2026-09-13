# Gamestruments Release Packet Template

Copy this file for one candidate and replace every blank or unchecked item with
literal evidence. The operator must approve the completed packet as a
single-use authorization before any itch.io transaction. Drafting this packet
is not approval. Agent simulation is supplementary and cannot satisfy human
acceptance or independent buyer acceptance.

## Release Identity
- **Version / Tag:**
- **Commit:**
- **Workflow Run URL:**
- **Godot Version:**
- **Rust Version:**
- **`RELEASE-MANIFEST.json` SHA-256:**
- **Manifest schema/product/version/provenance:**
- **Manifest source ref/commit/dirty:**

## Archive Information
- **Archive Filename:**
- **Archive Bytes:**
- **Archive SHA-256:**
- **Checksum sidecar filename and SHA-256:**
- **Retained rollback artifact location:**

### Supporting Downloads and HTML Preview

Build each item from the same final integrated commit as the kit, not an older
preview worktree. Record each upload separately in the approval and rollback plan.

| Item / destination | Filename | Bytes | SHA-256 | Source commit | Verification | Upload flags |
|---|---|---|---|---|---|---|
| Public docs + one example / paid product page | | | | | | Free demo file; no native binaries |
| Existing Audio Lab / `gamestruments-audio-lab-demo` | | | | | | Browser-playable HTML; record embed/mobile/fullscreen settings |

## Verification Evidence
- **Release workflow overall result:**

| Platform | Native job URL | Godot smoke result | Binary SHA-256 | Tester notes |
|---|---|---|---|---|
| Linux x86_64 / Ubuntu 24.04 | | | | |
| Windows x86_64 | | | | |
| macOS arm64 | | | | |
| macOS Intel x86_64 | | | | |

| Author QA platform | Tester | Date | Runbook result/evidence |
|---|---|---|---|
| Linux x86_64 / Ubuntu 24.04 | | | |
| Windows x86_64 | | | |
| macOS arm64 | | | |
| macOS Intel x86_64 | | | |

| Human listening platform | Listener | Date | Exact archive SHA verified | Result/notes |
|---|---|---|---|---|
| Linux x86_64 / Ubuntu 24.04 | | | | |
| Windows x86_64 | | | | |
| macOS arm64 | | | | |
| macOS Intel x86_64 | | | | |

### Independent Clean-Room Acceptance

- **Tester:**
- **Date / OS / architecture / Godot version:**
- **Exact archive SHA verified:**
- **Time to first sound:**
- **Time to first state transition:**

| Task from `docs/kit-plan.md` | Pass/fail | Evidence, failure, or unclear wording |
|---|---|---|
| Identify requirements from buyer docs | | |
| Run each packaged native example and reach audible output | | |
| Install root addon into a new project | | |
| Generate and check the result | | |
| Trigger all adaptive states | | |
| Confirm deterministic/suitably different seeds | | |
| Follow one troubleshooting path | | |
| Reconcile archive, limitations, and listing claims | | |

## Storefront Metadata (itch.io)
The complete candidate copy is `storefront/listing.md`. Record its Git blob or
SHA-256 so approval covers exact text.

- **`storefront/listing.md` identity:**
- **Title:** Gamestruments — Adaptive Music for Godot 4
- **Slug:** gamestruments-godot
- **Classification / kind:** Game Assets / Downloadable
- **Short description (exact approved text from listing):**
- **Price:** $12.99 (Minimum / Pay-what-you-want above)
- **Launch discount:** None
- **Language / release status:** English / Released
- **Tags (exact approved list):**
- **Payment Mode (operator decision; itch.io Payouts or Direct):**
- **Open Revenue Share percentage (operator decision):**
- **Upload Flags:** No OS executable flags (archive contains libraries/source).
- **Upload filename / size / SHA-256:**
- **Upload type selected in editor:**
- **Visibility after transaction:** Public
- **New downloads and purchases:** Enabled
- **Search/browse listing:** Enabled
- **Community/comments:** Enabled for public support
- **External links (record exact demo/docs links and navigation behavior):** Private repository URLs are prohibited.
- **Indexing expectation acknowledged:** First paid page may require itch.io manual review and indexing can lag publication.

### Exact Storefront Media

Native-audio evidence must come from the immutable packaged Godot kit. Record
which native example or approved Lab state each visual shows; do not present
browser playback as the exact native mix. Recordings from earlier drafts are stale.

| Use/order | Filename or video URL | SHA-256 if local | Dimensions/duration | Packaged-demo source state | Approved |
|---|---|---|---|---|---|
| Cover (315:250; 630x500 recommended) | | | | | |
| Screenshot 1 | | | | | |
| Screenshot 2 | | | | | |
| Screenshot 3 | | | | | |
| Screenshot 4 (optional) | | | | | |
| Screenshot 5 (optional) | | | | | |
| Music/captions/code video (required for this update's promotion) | | n/a | | | |

## Support and Disclosures
- **Support Route:** Public comments on the itch.io product page; purchase-specific or private matters use itch.io's purchase-support flow.
- **Refund Disclosures:** Standard itch.io terms at purchase time.
- **License disclosure:** First-party runtime/demo/docs MIT; dependency terms and attribution ship in the archive.

## Transaction and Rollback Plan
- **Exact transaction plan:** Create or edit the named page in Draft; enter only the approved fields; upload the exact archive directly to itch.io; apply the approved upload metadata; compare the page preview to this packet; switch to Public with purchases and search/browse enabled.
- **Abort condition:** Any field, media item, upload digest, price, payment setting, or visibility that differs from this packet requires a new approval.
- **Rollback:** Retain this archive and the immediately previous approved archive off-platform. For a bad upload, disable purchases or unlist first, then re-upload the retained prior artifact as the current file. Do not delete and recreate the page.
- **Retirement:** Existing owners retain access. Prefer disabling purchases/unlisting over deletion; assess buyer access and refunds before removing paid content.

## Single-Use Operator Approval

- **Approved page/account:**
- **Approved archive SHA-256:**
- **Approved public docs and HTML-preview SHA-256 values and destinations:**
- **Approved listing identity:**
- **Approved media identities:**
- **Approved payment mode/revenue share/price:**
- **Approved visibility/final action:**
- **Operator / timestamp:**
- **Consumed by transaction / timestamp / resulting URL:**

*For official itch creator documentation, see: https://itch.io/docs/creators/*
