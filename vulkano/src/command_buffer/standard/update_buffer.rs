// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::mem;
use std::ops::Range;
use std::sync::Arc;

use buffer::Buffer;
use buffer::BufferSlice;
use command_buffer::standard::StdCommandBufferBuilder;
use command_buffer::sys::UnsafeCommandBufferBuilder;
use image::Image;
use image::sys::Layout;
use sync::AccessFlagBits;
use sync::PipelineStages;

/// Wrapper around a `StdCommandBufferBuilder` that adds a buffer updating command at the end of
/// the builder.
pub struct StdUpdateBufferBuilder<'a, T, D: 'a, B> {
    inner: T,
    data: &'a D,
    buffer: Arc<B>,
}

impl<'a, T, D, B> StdUpdateBufferBuilder<'a, T, D, B> where T: StdCommandBufferBuilder {
    /// Adds the command at the end of `inner`.
    pub fn new<'b, S>(mut inner: T, buffer: S, data: &'a D) -> StdUpdateBufferBuilder<'a, T, D, B>
        where S: Into<BufferSlice<'b, D, B>>,
              B: Buffer + 'b,
              D: Copy + 'static,
    {
        let buffer = buffer.into();

        // FIXME: check outsideness of render pass

        // TODO: return error instead
        assert_eq!(buffer.offset() % 4, 0);
        assert_eq!(buffer.size() % 4, 0);
        assert!(mem::size_of_val(data) <= 65536);
        assert!(buffer.buffer().inner_buffer().usage_transfer_dest());

        let keep_alive = buffer.buffer().clone();

        let stage = PipelineStages {
            transfer: true,
            .. PipelineStages::none()
        };

        let access = AccessFlagBits {
            transfer_write: true,
            .. AccessFlagBits::none()
        };

        unsafe {
            inner.add_buffer_usage(&buffer.buffer(),
                                   buffer.offset() .. (buffer.offset() + buffer.size()),
                                   true, stage, access);

            inner.add_command(move |cb| cb.update_buffer(buffer, data));
        }

        StdUpdateBufferBuilder {
            inner: inner,
            data: data,
            buffer: keep_alive,
        }
    }
}

unsafe impl<'a, T, D: 'a, B> StdCommandBufferBuilder for StdUpdateBufferBuilder<'a, T, D, B>
    where T: StdCommandBufferBuilder,
          B: Buffer
{
    type BuildOutput = T::BuildOutput;
    type Pool = T::Pool;

    #[inline]
    unsafe fn add_command<F>(&mut self, cmd: F)
        where F: FnOnce(&mut UnsafeCommandBufferBuilder<Self::Pool>)
    {
        self.inner.add_command(cmd)
    }

    #[inline]
    unsafe fn add_buffer_usage<X>(&mut self, buffer: &Arc<X>, slice: Range<usize>, write: bool,
                                  stages: PipelineStages, accesses: AccessFlagBits)
        where X: Buffer
    {
        self.inner.add_buffer_usage(buffer, slice, write, stages, accesses)
    }

    #[inline]
    unsafe fn add_image_usage<I>(&mut self, image: &Arc<I>, mipmaps: Range<u32>,
                                 array_layers: Range<u32>, write: bool, layout: Layout,
                                 stages: PipelineStages, accesses: AccessFlagBits)
        where I: Image
    {
        self.inner.add_image_usage(image, mipmaps, array_layers, write, layout, stages, accesses)
    }

    #[inline]
    fn build(self) -> Self::BuildOutput {
        self.inner.build()
    }
}
