# Security policy

## Supported versions

The crate is pre-1.0. Only the most recent `0.x` release receives security fixes.

## Reporting a vulnerability

Do not report vulnerabilities in a public issue, pull request or discussion.

Use GitHub's private vulnerability reporting: open the repository's **Security** tab and choose **Report a
vulnerability**. Include:

- what the problem is and where it is (file, function or widget),
- the steps or code that reproduce it,
- the version or commit you tested,
- what an attacker or a user could do with it.

The maintainer replies through the same private thread, and coordinates a fix and a disclosure date with you.

## Scope

This crate is a library for egui user interfaces. It contains no network code and no file I/O. Reports that
matter most are:

- memory or resource exhaustion caused by input that the widgets render (very long text, very large grids),
- unexpected data leaving the application through a widget (for example the inspector's copy button),
- problems in the embedded font or icon data.

The crate forbids `unsafe` code with a lint, so memory-safety issues are unlikely; a report that shows one is
still welcome.

Problems in egui, eframe or other dependencies should be reported to their upstream projects. Tell us as well
if the problem affects this crate's use of them.
