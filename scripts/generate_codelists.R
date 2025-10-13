codelist <- kokudosuuchi:::.codelist
codes_sanitized <- chartr("-.", "__", names(codelist))
names(codelist) <- codes_sanitized

output_path <- "rust/src/translate/data/codelists.rs"

enum <- c(
  "pub(crate) enum ColTypes {",
  paste0("    ", codes_sanitized, ","),
  "}"
)

map <- purrr::imap(codelist, \(df, codelist_name) {
  c(
    "#[rustfmt::skip]",
    glue::glue("pub(crate) const {codelist_name}: &[(&str, &str)] = &["),
    glue::glue_data(df, '  ("{code}", "{label}"),'),
    "];\n"
  )
}) |>
  purrr::flatten_chr()

setwd("~/GitHub/ksj2gp/")
brio::write_lines(c(enum, map), output_path)
