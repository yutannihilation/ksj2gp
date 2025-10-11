output_path <- "rust/src/translate/data/colnames.rs"

d <- kokudosuuchi:::.col_info$other
ids <- unique(d$id)

result <- "#[rustfmt::skip]
pub(crate) const COLNAMES: &[(&str, &str)] = &["

for (id in ids) {
  result <- append(result, glue::glue("\n  ///// {id} /////\n", .trim = FALSE))
  d_tmp <- d |> 
    dplyr::filter(id == {{ id }})

  result <- append(
    result,
    glue::glue_data(d_tmp, '  ("{code}", "{name}"),')
  )
}

result <- append(result, "\n];")

brio::write_lines(result, output_path)