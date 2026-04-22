pub(super) struct StepMeta {
    pub(super) script: &'static str,
    pub(super) label: &'static str,
    pub(super) narrative: &'static str,
    pub(super) outcome: &'static str,
}

pub(super) struct CategoryMeta {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) tagline: &'static str,
    pub(super) story: &'static str,
    pub(super) cost: &'static str,
    pub(super) feature_url: &'static str,
    pub(super) steps: &'static [StepMeta],
}

pub(super) struct PillarMeta {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) feature_url: &'static str,
    pub(super) category_ids: &'static [&'static str],
}
