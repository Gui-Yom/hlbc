use std::time::Instant;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{
    Field, IndexRecordOption, NumericOptions, Schema, TextFieldIndexing, TextOptions, STORED,
};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer};
use tantivy::{doc, Index};

use hlbc::types::RefFun;
use hlbc::Bytecode;

mod tokenizer;

struct TantivySearcher {
    index: Index,
}

impl TantivySearcher {
    pub fn new_functions(code: &Bytecode) -> Self {
        // let code_tokenizer = TextAnalyzer::from(FunctionTokenizer)
        //     .filter(LowerCaser)
        //     .filter(Stemmer::new(Language::English));
        let code_tokenizer =
            TextAnalyzer::from(NgramTokenizer::all_ngrams(3, 10)).filter(LowerCaser);

        let mut schema_builder = Schema::builder();
        let findex = schema_builder.add_u64_field("findex", NumericOptions::default() | STORED);
        let name = schema_builder.add_text_field(
            "name",
            TextOptions::default().set_indexing_options(
                TextFieldIndexing::default()
                    .set_fieldnorms(true)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions)
                    .set_tokenizer("function"),
            ),
        );
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());
        index.tokenizers().register("function", code_tokenizer);

        let mut writer = index.writer_with_num_threads(1, 10_000_000).unwrap();
        let start = Instant::now();
        for f in code.functions() {
            writer
                .add_document(doc!(
                    findex => f.findex().0 as u64,
                    name => &*f.name(code)
                ))
                .unwrap();
        }
        writer.commit().unwrap();
        println!(
            "Indexed all documents in {} ms",
            start.elapsed().as_millis()
        );

        Self { index }
    }
}

impl TantivySearcher {
    pub fn query(&self, query_text: &str) -> Vec<RefFun> {
        let reader = self.index.reader().unwrap();
        let searcher = reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![Field::from_field_id(1)]);
        let query = parser.parse_query(query_text).unwrap();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
        top_docs
            .into_iter()
            .map(|(_, d)| {
                RefFun(
                    searcher
                        .doc(d)
                        .unwrap()
                        .get_first(Field::from_field_id(0))
                        .unwrap()
                        .as_u64()
                        .unwrap() as usize,
                )
            })
            .collect()
    }
}
