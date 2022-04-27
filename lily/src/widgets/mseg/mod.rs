//! Multi-stage envelope generator widget

pub(crate) mod graph;
pub(crate) mod util;

use self::graph::{MsegGraph, MsegGraphHandle};
use std::{marker::PhantomData, ops::RangeInclusive};

use super::zoomer::{Zoomer, ZoomerHandle};
use crate::util::CurvePoints;
use glam::Vec2;
use lily_derive::Handle;
use vizia::*;

#[allow(clippy::enum_variant_names)]
enum MsegInternalEvent {
    OnChangingRangeStart(f32),
    OnChangingRangeEnd(f32),
    OnChangingRangeBoth { start: f32, end: f32 },
    OnChangingPoint { index: usize, point: Vec2 },
    OnRemovePoint { index: usize },
    OnInsertPoint { index: usize, point: Vec2 },
}

#[allow(clippy::type_complexity)]
#[derive(Handle)]
pub struct Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    points: P,
    range: PhantomData<R>,

    #[callback(usize)]
    on_remove_point: Option<Box<dyn Fn(&mut Context, usize)>>,

    #[callback(usize, Vec2)]
    on_insert_point: Option<Box<dyn Fn(&mut Context, usize, Vec2)>>,

    #[callback(usize, Vec2)]
    on_changing_point: Option<Box<dyn Fn(&mut Context, usize, Vec2)>>,

    #[callback(f32)]
    on_changing_range_start: Option<Box<dyn Fn(&mut Context, f32)>>,

    #[callback(f32)]
    on_changing_range_end: Option<Box<dyn Fn(&mut Context, f32)>>,

    #[callback(RangeInclusive<f32>)]
    on_changing_range_both: Option<Box<dyn Fn(&mut Context, RangeInclusive<f32>)>>,
}

impl<P, R> Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(cx: &mut Context, points: P, range: R, max: f32) -> Handle<Mseg<P, R>> {
        Self {
            points: points.clone(),
            range: Default::default(),
            on_changing_point: None,
            on_changing_range_start: None,
            on_changing_range_end: None,
            on_changing_range_both: None,
            on_remove_point: None,
            on_insert_point: None,
        }
        .build(cx, |cx| {
            MsegGraph::new(cx, points, range.clone(), max)
                .on_changing_point(|cx, index, point| {
                    cx.emit(MsegInternalEvent::OnChangingPoint { index, point })
                })
                .on_remove_point(|cx, index| cx.emit(MsegInternalEvent::OnRemovePoint { index }))
                .on_insert_point(|cx, index, point| {
                    cx.emit(MsegInternalEvent::OnInsertPoint { index, point })
                })
                .class("graph");

            Zoomer::new(cx, range.clone())
                .on_changing_start(|cx, x| cx.emit(MsegInternalEvent::OnChangingRangeStart(x)))
                .on_changing_end(|cx, x| cx.emit(MsegInternalEvent::OnChangingRangeEnd(x)))
                .on_changing_both(|cx, start, end| {
                    cx.emit(MsegInternalEvent::OnChangingRangeBoth { start, end })
                });
        })
    }
}

impl<P, R> View for Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn element(&self) -> Option<String> {
        Some("mseg".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|ev: &MsegInternalEvent, _| match *ev {
            MsegInternalEvent::OnChangingRangeStart(x) => {
                if let Some(callback) = &self.on_changing_range_start {
                    (callback)(cx, x);
                }
            }
            MsegInternalEvent::OnChangingRangeEnd(x) => {
                if let Some(callback) = &self.on_changing_range_end {
                    (callback)(cx, x);
                }
            }
            MsegInternalEvent::OnChangingRangeBoth { start, end } => {
                if let Some(callback) = &self.on_changing_range_both {
                    (callback)(cx, start..=end);
                }
            }
            MsegInternalEvent::OnChangingPoint { index, point } => {
                if let Some(callback) = &self.on_changing_point {
                    (callback)(cx, index, point);
                }
            }
            MsegInternalEvent::OnRemovePoint { index } => {
                // Delete the point if not the first or last in the vector
                if index != 0 && index != self.points.get(cx).len() - 1 {
                    if let Some(callback) = &self.on_remove_point {
                        (callback)(cx, index);
                    }
                }
            }
            MsegInternalEvent::OnInsertPoint { index, point } => {
                if let Some(callback) = &self.on_insert_point {
                    (callback)(cx, index, point);
                }
            }
        });
    }
}
