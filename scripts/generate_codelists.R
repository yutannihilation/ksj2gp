codelist <- kokudosuuchi:::.codelist
codelist_id_sanitized <- chartr("-.", "__", names(codelist))
names(codelist) <- codelist_id_sanitized

output_path <- "rust/src/translate/data/codelists.rs"

enum <- c(
  "pub(crate) enum CodelistId {",
  paste0("    ", codelist_id_sanitized, ","),
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

brio::write_lines(c(enum, map), output_path)
