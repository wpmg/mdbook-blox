use super::blox::Blox;
use super::figure::Figure;
use super::leaf::{Leaf, content_to_leafs_excl_reference};
use super::number_map::NumberMap;
use crate::config::FIGURE_BLOCK_KEYWORD;
use crate::config::processor_config::Config;
use crate::render::Render;
use anyhow::{Context, Result, bail};
use mdbook_preprocessor::book::Chapter;
use std::borrow::{Borrow, BorrowMut, Cow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use uuid::Uuid;

#[derive(Debug)]
pub struct Collection<'a> {
    config: &'a Config,
    bloxes: BloxCollection,
    texts: TextCollection,
    figures: FigureCollection,
    order: OrderCollection,
}

impl<'a> Collection<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            bloxes: BloxCollection::default(),
            texts: TextCollection::default(),
            figures: FigureCollection::default(),
            order: OrderCollection::default(),
        }
    }

    #[inline]
    pub fn bloxes(&self) -> &BloxCollection {
        &self.bloxes
    }
    #[inline]
    pub fn figures(&self) -> &FigureCollection {
        &self.figures
    }

    pub fn process_collections(&mut self) -> Result<()> {
        let mut number_map = NumberMap::new(self.config);

        for (_, chapter) in self.order.hash_map_iter() {
            number_map.reset(chapter.number().map(|n| n.to_string()));

            for ocid in chapter.order().iter() {
                //
                // Process blox paths et c.
                //
                if let CollectionId::Blox(ref blox_id) = ocid.id {
                    self.bloxes
                        .set_from_section(blox_id, &mut number_map, chapter.path())?;
                }

                //
                // Process figures
                //
                let Some(content) = (match ocid.id() {
                    CollectionId::Blox(v) => self.bloxes.content_mut(v),
                    CollectionId::Text(v) => self.texts.content_mut(*v),
                }) else {
                    continue;
                };

                let leafs = content_to_leafs_excl_reference(FIGURE_BLOCK_KEYWORD, content)?;

                if leafs.is_empty()
                    || (leafs.len() == 1 && (leafs[0].is_text() || leafs[0].is_none()))
                {
                    continue;
                }

                let new_content: String = leafs
                    .into_iter()
                    .filter_map(|leaf| match leaf {
                        l @ Leaf::Blox { .. } => self
                            .figures
                            .push_leaf(l, self.config, &mut number_map, chapter.path())
                            .ok(),
                        Leaf::Text { content, .. } => Some(content.to_string()),
                        _ => None,
                    })
                    .collect();

                *content = new_content;
            }
        }

        Ok(())
    }

    // CONSTRUCTION
    pub fn push_raw_leaf(
        &mut self,
        leaf: Leaf<'_>,
        chapter: &Chapter,
        config: &Config,
    ) -> Result<()> {
        let ocid_opt: Option<OrderedCollectionId> = match leaf {
            Leaf::Text { range, content } => {
                let id = self.texts.push(content);
                Some(OrderedCollectionId::new(id, range.start))
            }
            Leaf::Blox { ref range, .. } => {
                let start = range.start;
                let blox = Blox::from_leaf(leaf, config)?;
                let defer = blox
                    .blox_label()
                    .map(|l| l.defer_rendering())
                    .unwrap_or(false);
                let id = self.bloxes.push(blox);

                // If the blox is deferred, we shouldn't return a leaf.
                // Otherwise, the leaf must be returned.
                (!defer).then(|| OrderedCollectionId::new(id, start))
            }
            Leaf::BloxReference { range, label } => Some(OrderedCollectionId::new(
                CollectionId::Blox(label),
                range.start,
            )),
            _ => None,
        };

        self.order.push(chapter, ocid_opt)
    }

    // RENDER
    pub fn section_to_string(&self, path_str: &str) -> String {
        let new_content: String = self
            .order
            .iter(path_str)
            .map(|ocid| match ocid.id() {
                CollectionId::Blox(v) => self.bloxes.render(v, self.config),
                CollectionId::Text(v) => self.texts.to_string(*v),
            })
            .collect();

        new_content
    }
}

#[derive(Default, Debug)]
pub struct BloxCollection(HashMap<String, Blox>);
impl BloxCollection {
    #[inline]
    pub fn get(&self, id: &str) -> Option<&Blox> {
        self.0.get(id)
    }
    #[inline]
    fn get_mut(&mut self, id: &str) -> Option<&mut Blox> {
        self.0.get_mut(id)
    }
    // #[inline]
    // fn content(&self, id: &str) -> Option<&str> {
    //     self.get(id).map(|b| b.content())
    // }
    #[inline]
    fn content_mut(&mut self, id: &str) -> Option<&mut String> {
        self.get_mut(id).map(|b| b.content_mut())
    }
    #[inline]
    fn render(&self, id: &str, config: &Config) -> String {
        self.get(id)
            .map(|b| match config.renderer() {
                "pandoc" => b.latex(config),
                _ => b.html(config),
            })
            .unwrap_or_default()
    }

    fn push(&mut self, blox: Blox) -> CollectionId {
        let label = blox
            .label()
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());

        if self.0.insert(label.clone(), blox).is_some() {
            tracing::error!("Blox labels collision: {label}");
        }

        CollectionId::Blox(label)
    }

    fn set_from_section(
        &mut self,
        id: &str,
        number_map: &mut NumberMap,
        path: &Path,
    ) -> Result<()> {
        let blox = self
            .get_mut(id)
            .context(format!("Could not find blox: {}", id))?;
        blox.set_number(|env| number_map.next_string(env))?;
        blox.set_path(path);
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct TextCollection(Vec<String>);
impl TextCollection {
    #[inline]
    fn content(&self, id: usize) -> Option<&str> {
        self.0.get(id).map(|t| t.borrow())
    }
    #[inline]
    fn content_mut(&mut self, id: usize) -> Option<&mut String> {
        self.0.get_mut(id).map(|t| t.borrow_mut())
    }
    #[inline]
    fn to_string(&self, id: usize) -> String {
        self.content(id).map(|t| t.to_string()).unwrap_or_default()
    }

    fn push(&mut self, text: Cow<'_, str>) -> CollectionId {
        self.0.push(text.to_string());
        CollectionId::Text(self.0.len() - 1)
    }
}

#[derive(Default, Debug)]
pub struct FigureCollection(HashMap<String, Figure>);
impl FigureCollection {
    #[inline]
    pub fn get(&self, id: &str) -> Option<&Figure> {
        self.0.get(id)
    }
    // #[inline]
    // fn get_mut(&mut self, id: &str) -> Option<&mut Figure> {
    //     self.0.get_mut(id)
    // }
    // #[inline]
    // fn content(&self, ocid: &str) -> Option<&str> {
    //     self.get(ocid).map(|b| b.content())
    // }

    fn push(&mut self, figure: Figure) {
        let label = figure.label();

        if !label.is_empty() {
            self.0.insert(label.to_string(), figure);
        }
    }

    fn push_leaf(
        &mut self,
        leaf: Leaf,
        config: &Config,
        number_map: &mut NumberMap,
        path: &Path,
    ) -> Result<String> {
        let mut fig = Figure::from_leaf(leaf, config)?;

        // Handle numbering et c.
        fig.set_number(|| Ok(number_map.next_figure_string()))?;
        fig.set_path(path);

        let s = match config.renderer() {
            "pandoc" => fig.latex(config),
            _ => fig.html(config),
        };

        self.push(fig);

        Ok(s)
    }
}

#[derive(Default, Debug)]
/// path // info combo
pub struct OrderCollection(HashMap<String, ChapterInfo>);
impl OrderCollection {
    #[inline]
    pub fn hash_map_iter(&self) -> impl Iterator<Item = (&String, &ChapterInfo)> {
        self.0.iter()
    }
    #[inline]
    pub fn get(&self, path_str: &str) -> Option<&Vec<OrderedCollectionId>> {
        self.0.get(path_str).map(|ci| ci.order())
    }
    #[inline]
    pub fn iter(&self, path_str: &str) -> Iter<'_, OrderedCollectionId> {
        self.0
            .get(path_str)
            .map(|v| v.order().iter())
            .unwrap_or_default()
    }
    #[inline]
    pub fn blox_iter(&self, path_str: &str) -> impl Iterator<Item = &str> {
        self.iter(path_str).filter_map(|ocid| match ocid.id {
            CollectionId::Blox(ref v) => Some(v.as_str()),
            _ => None,
        })
    }

    fn push(&mut self, chapter: &Chapter, ocid: Option<OrderedCollectionId>) -> Result<()> {
        let Some(v) = ocid else {
            return Ok(());
        };

        let path_str = ChapterInfo::path_to_str(chapter)?;

        if let Some(arr) = self.0.get_mut(path_str) {
            arr.order_mut().push(v);
        } else {
            self.0
                .insert(path_str.to_string(), ChapterInfo::new(chapter, v)?);
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ChapterInfo {
    order: Vec<OrderedCollectionId>,
    number: Option<String>,
    path: PathBuf,
}

impl ChapterInfo {
    #[inline]
    pub fn new(chapter: &Chapter, ocid: OrderedCollectionId) -> Result<Self> {
        let Some(path) = chapter.path.clone() else {
            bail!("Chapter does not have path");
        };

        Ok(Self {
            order: vec![ocid],
            number: chapter.number.as_ref().map(|n| n.to_string()),
            path,
        })
    }
    #[inline]
    pub fn order(&self) -> &Vec<OrderedCollectionId> {
        &self.order
    }
    #[inline]
    pub fn order_mut(&mut self) -> &mut Vec<OrderedCollectionId> {
        &mut self.order
    }
    #[inline]
    pub fn number(&self) -> Option<&str> {
        self.number.as_deref()
    }
    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    #[inline]
    pub fn path_to_str(chapter: &Chapter) -> Result<&str> {
        chapter
            .path
            .as_ref()
            .and_then(|p| p.to_str())
            .context("Chapter does not have a valid unicode path")
    }
}

#[derive(Debug, Clone)]
pub enum CollectionId {
    Blox(String),
    Text(usize),
}

#[derive(Debug, Clone)]
pub struct OrderedCollectionId {
    start: usize,
    id: CollectionId,
}

impl OrderedCollectionId {
    // GETTERS
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }
    #[inline]
    pub fn id(&self) -> &CollectionId {
        &self.id
    }

    #[inline]
    pub fn is_blox(&self) -> bool {
        matches!(self.id, CollectionId::Blox(_))
    }

    // CONSTRUCTORS
    #[inline]
    fn new(id: CollectionId, start: usize) -> Self {
        Self { start, id }
    }
}
