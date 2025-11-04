use super::blox::Blox;
use super::figure::Figure;
use super::leaf::Leaf;
use super::number_map::NumberMap;
use crate::blox::leaf::content_to_leafs_excl_reference;
use crate::config::FIGURE_BLOCK_KEYWORD;
use crate::config::processor_config::Config;
use crate::render::Render;
use anyhow::{Context, Result};
use std::borrow::{Borrow, BorrowMut, Cow};
use std::collections::HashMap;
use std::path::PathBuf;
use std::slice::Iter;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct Collection<'a> {
    bloxes: BloxCollection<'a>,
    texts: TextCollection<'a>,
    figures: FigureCollection,
    order: OrderCollection,
}

impl<'a> Collection<'a> {
    #[inline]
    pub fn bloxes(&self) -> &BloxCollection<'a> {
        &self.bloxes
    }
    #[inline]
    pub fn figures(&self) -> &FigureCollection {
        &self.figures
    }

    pub fn set_blox(
        &mut self,
        section_id: usize,
        number_map: &mut NumberMap,
        path: Option<&PathBuf>,
    ) -> Result<()> {
        for id in self.order.blox_iter(section_id) {
            self.bloxes.set_from_section(id, number_map, path)?;
        }

        Ok(())
    }

    pub fn process_figures(
        &mut self,
        config: &Config,
        section_id: usize,
        number_map: &mut NumberMap,
        path: Option<&PathBuf>,
    ) -> Result<()> {
        for ocid in self.order.iter(section_id) {
            let Some(content) = (match ocid.id() {
                CollectionId::Blox(v) => self.bloxes.content_mut(v),
                CollectionId::Text(v) => self.texts.content_mut(*v),
            }) else {
                continue;
            };

            let leafs = content_to_leafs_excl_reference(FIGURE_BLOCK_KEYWORD, content)?;

            if leafs.len() == 0 || (leafs.len() == 1 && (leafs[0].is_text() || leafs[0].is_none()))
            {
                continue;
            }

            let new_content: String = leafs
                .into_iter()
                .filter_map(|leaf| match leaf {
                    l @ Leaf::Blox { .. } => {
                        self.figures.push_leaf(l, config, number_map, path).ok()
                    }
                    Leaf::Text { content, .. } => Some(content.to_string()),
                    _ => None,
                })
                .collect();

            *content = Cow::Owned(new_content);
        }

        Ok(())
    }

    // CONSTRUCTION
    pub fn push_raw_leaf(
        &mut self,
        leaf: Leaf<'a>,
        section_id: usize,
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

        self.order.push(section_id, ocid_opt)
    }

    // RENDER
    pub fn section_to_string(&self, config: &Config, section_id: usize) -> String {
        let new_content: String = self
            .order
            .iter(section_id)
            .map(|ocid| match ocid.id() {
                CollectionId::Blox(v) => self.bloxes.render(v, config),
                CollectionId::Text(v) => self.texts.to_string(*v),
            })
            .collect();

        new_content
    }
}

#[derive(Default, Debug)]
pub struct BloxCollection<'a>(HashMap<String, Blox<'a>>);
impl<'a> BloxCollection<'a> {
    #[inline]
    pub fn get(&self, id: &str) -> Option<&Blox<'a>> {
        self.0.get(id)
    }
    #[inline]
    fn get_mut(&mut self, id: &str) -> Option<&mut Blox<'a>> {
        self.0.get_mut(id)
    }
    // #[inline]
    // fn content(&self, id: &str) -> Option<&str> {
    //     self.get(id).map(|b| b.content())
    // }
    #[inline]
    fn content_mut(&mut self, id: &str) -> Option<&mut Cow<'a, str>> {
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

    fn push(&mut self, blox: Blox<'a>) -> CollectionId {
        let label = blox
            .label()
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());

        if self.0.insert(label.clone(), blox).is_some() {
            log::error!("Blox labels collision: {label}");
        }

        CollectionId::Blox(label)
    }

    fn set_from_section(
        &mut self,
        id: &str,
        number_map: &mut NumberMap,
        path: Option<&PathBuf>,
    ) -> Result<()> {
        let blox = self.get_mut(id).context("Could not find blox")?;
        blox.set_number(|env| number_map.next_string(env))?;
        blox.set_path(path);
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct TextCollection<'a>(Vec<Cow<'a, str>>);
impl<'a> TextCollection<'a> {
    #[inline]
    fn content(&self, id: usize) -> Option<&str> {
        self.0.get(id).map(|t| t.borrow())
    }
    #[inline]
    fn content_mut(&mut self, id: usize) -> Option<&mut Cow<'a, str>> {
        self.0.get_mut(id).map(|t| t.borrow_mut())
    }
    #[inline]
    fn to_string(&self, id: usize) -> String {
        self.content(id).map(|t| t.to_string()).unwrap_or_default()
    }

    fn push(&mut self, text: Cow<'a, str>) -> CollectionId {
        self.0.push(text);
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
        path: Option<&PathBuf>,
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
pub struct OrderCollection(HashMap<usize, Vec<OrderedCollectionId>>);
impl OrderCollection {
    #[inline]
    pub fn get(&self, section_id: usize) -> Option<&Vec<OrderedCollectionId>> {
        self.0.get(&section_id)
    }
    #[inline]
    pub fn iter(&self, section_id: usize) -> Iter<'_, OrderedCollectionId> {
        self.0
            .get(&section_id)
            .map(|v| v.iter())
            .unwrap_or_default()
    }
    #[inline]
    pub fn blox_iter(&self, section_id: usize) -> impl Iterator<Item = &str> {
        self.0
            .get(&section_id)
            .map(|v| v.iter())
            .unwrap_or_default()
            .filter_map(|ocid| match ocid.id {
                CollectionId::Blox(ref v) => Some(v.as_str()),
                _ => None,
            })
    }

    fn push(&mut self, section_id: usize, ocid: Option<OrderedCollectionId>) -> Result<()> {
        let Some(v) = ocid else {
            return Ok(());
        };

        if let Some(arr) = self.0.get_mut(&section_id) {
            arr.push(v);
        } else {
            self.0.insert(section_id, vec![v]);
        }

        Ok(())
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
        match self.id {
            CollectionId::Blox(_) => true,
            _ => false,
        }
    }

    // CONSTRUCTORS
    #[inline]
    fn new(id: CollectionId, start: usize) -> Self {
        Self { start, id }
    }
}
