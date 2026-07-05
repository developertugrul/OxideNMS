use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    Inserted,
    Deleted,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct DiffRow {
    pub tip: DiffType,
    pub old_line_no: Option<usize>,
    pub new_line_no: Option<usize>,
    pub text: String,
}

/// İki config metnini satır bazında karşılaştırır.
pub fn compare_configs(eski: &str, yeni: &str) -> Vec<DiffRow> {
    let diff = TextDiff::from_lines(eski, yeni);
    let mut result = Vec::new();

    for change in diff.iter_all_changes() {
        let tip = match change.tag() {
            ChangeTag::Delete => DiffType::Deleted,
            ChangeTag::Insert => DiffType::Inserted,
            ChangeTag::Equal => DiffType::Unchanged,
        };

        result.push(DiffRow {
            tip,
            old_line_no: change.old_index().map(|i| i + 1),
            new_line_no: change.new_index().map(|i| i + 1),
            text: change.value().to_string(),
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_bulma_testi() {
        let eski = "hostname R1\n!\ninterface G0/0\n shutdown\n";
        let yeni = "hostname R1\n!\ninterface G0/0\n no shutdown\n";

        let fark = compare_configs(eski, yeni);

        // 4 satır var. 1, 2, 3 değişmedi. 4 silindi, 4 eklendi. (toplam 5 eleman döner)
        assert_eq!(fark.len(), 5);
        assert_eq!(fark[0].tip, DiffType::Unchanged);
        assert_eq!(fark[3].tip, DiffType::Deleted);
        assert_eq!(fark[3].text, " shutdown\n");
        assert_eq!(fark[4].tip, DiffType::Inserted);
        assert_eq!(fark[4].text, " no shutdown\n");
    }
}
