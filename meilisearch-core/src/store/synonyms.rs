use std::borrow::Cow;

use deunicode::deunicode;
use heed::Result as ZResult;
use heed::types::{ByteSlice, Str, SerdeBincode};

use crate::database::MainT;
use crate::{FstSetCow, MResult};

#[derive(Copy, Clone)]
pub struct Synonyms {
    pub(crate) synonyms: heed::Database<ByteSlice, ByteSlice>,
    pub(crate) synonyms_unicased: heed::Database<Str, SerdeBincode<Vec<String>>>,
}

impl Synonyms {
    pub fn put_synonyms<A>(self, writer: &mut heed::RwTxn<MainT>, word: &str, synonyms: &fst::Set<A>) -> MResult<()>
    where A: AsRef<[u8]>,
    {
        let deunicoded = deunicode(word);
        let bytes = synonyms.as_fst().as_bytes();
        self.synonyms.put(writer, deunicoded.as_bytes(), bytes)?;
        let synonyms_vec = synonyms.stream().into_strs()?;
        self.synonyms_unicased.put(writer, word, &synonyms_vec)?;
        Ok(())
    }

    pub fn del_synonyms(self, writer: &mut heed::RwTxn<MainT>, word: &str) -> ZResult<bool> {
        let deunicoded = deunicode(word);
        self.synonyms_unicased.delete(writer, word)?;
        self.synonyms.delete(writer, deunicoded.as_bytes())
    }

    pub fn clear(self, writer: &mut heed::RwTxn<MainT>) -> ZResult<()> {
        self.synonyms.clear(writer)?;
        self.synonyms_unicased.clear(writer)?;
        Ok(())
    }

    pub(crate) fn synonyms_fst<'txn>(self, reader: &'txn heed::RoTxn<MainT>, word: &str) -> ZResult<FstSetCow<'txn>> {
        let deunicoded = deunicode(word);
        match self.synonyms.get(reader, deunicoded.as_bytes())? {
            Some(bytes) => Ok(fst::Set::new(bytes).unwrap().map_data(Cow::Borrowed).unwrap()),
            None => Ok(fst::Set::default().map_data(Cow::Owned).unwrap()),
        }
    }

    pub fn synonyms_unicased(self, reader: &heed::RoTxn<MainT>, word: &str) -> MResult<Vec<String>> {
        Ok(self.synonyms_unicased.get(reader, word)?.unwrap_or_default())
    }

    pub fn synonyms(self, reader: &heed::RoTxn<MainT>, word: &str) -> MResult<Vec<String>> {
        let synonyms = self
            .synonyms_fst(&reader, word)?
            .stream()
            .into_strs()?;
        Ok(synonyms)
    }
}
