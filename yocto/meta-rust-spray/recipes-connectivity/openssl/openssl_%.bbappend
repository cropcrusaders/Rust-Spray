# Disable building of OpenSSL tests and fuzzers to avoid link errors during cross compile
EXTRA_OECONF:append = " no-tests no-fuzz"
