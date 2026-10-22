use quick_xml::events::BytesRef;

pub fn reconstruct_entity_ref(br: &BytesRef<'_>) -> Vec<u8> {
    std::iter::once(b'&')
        .chain(br.iter().copied())
        .chain(std::iter::once(b';'))
        .collect()
}
