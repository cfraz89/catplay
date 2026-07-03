use std::ptr;

use bytes::BytesMut;
use catplay_tracing_macro::trace_time;
use log::debug;
use x264_sys::*;

use crate::{H264Encoder, H264FrameBufferError, YuvShadowBuffer};

pub struct X264FrameBuffer {
    encoder: *mut x264_t,
    yuv: YuvShadowBuffer,
}

unsafe impl Send for X264FrameBuffer {}

impl X264FrameBuffer {
    pub fn new(width: i32, height: i32) -> Result<Self, H264FrameBufferError> {
        let mut param: x264_param_t = unsafe { std::mem::zeroed() };
        let ret = unsafe { x264_param_default_preset(&mut param, c"ultrafast".as_ptr() as _, c"zerolatency".as_ptr() as _) };

        if ret < 0 {
            return Err(H264FrameBufferError::X264DefaultPreset(ret));
        }

        param.i_keyint_max = 1;
        param.i_keyint_min = 1;
        param.i_scenecut_threshold = 0;
        param.b_intra_refresh = 0;
        param.i_bframe = 0;
        param.i_bframe_adaptive = 0;
        param.i_frame_reference = 1;
        param.i_slice_count = 1;
        param.i_slice_count_max = 1;
        param.i_dpb_size = 1;

        param.b_cabac = 0;
        param.b_aud = 0;
        param.b_repeat_headers = 0;
        param.b_annexb = 1;
        param.b_deblocking_filter = 0;
        param.b_open_gop = 0;
        param.b_vfr_input = 0;

        param.i_sync_lookahead = 0;
        param.rc.i_lookahead = 0;
        param.rc.i_rc_method = X264_RC_CQP as _;
        param.rc.i_qp_constant = 23;
        param.rc.b_mb_tree = 0;
        param.rc.i_aq_mode = 0;

        param.i_width = width;
        param.i_height = height;
        param.i_csp = X264_CSP_I420 as _;
        param.i_bframe_pyramid = 0;

        param.analyse.intra = 0;
        param.analyse.inter = 0;
        param.analyse.i_me_method = 0;
        param.analyse.i_subpel_refine = 0;
        param.analyse.i_trellis = 0;
        param.analyse.b_transform_8x8 = 0;
        param.analyse.b_psy = 0;
        param.analyse.b_psnr = 0;
        param.analyse.b_ssim = 0;
        param.analyse.i_direct_mv_pred = 0;
        param.analyse.b_chroma_me = 0;
        param.analyse.b_fast_pskip = 0;
        param.analyse.b_dct_decimate = 0;
        param.analyse.b_mixed_references = 0;

        param.i_threads = 1;
        param.i_lookahead_threads = 0;
        param.b_sliced_threads = 0;

        debug!("x264 params {param:?}");

        let ret = unsafe { x264_param_apply_profile(&mut param, c"baseline".as_ptr() as _) };
        if ret < 0 {
            return Err(H264FrameBufferError::X264ApplyProfile(ret));
        }

        let encoder = unsafe { x264_encoder_open(&mut param) };
        if encoder.is_null() {
            return Err(H264FrameBufferError::X264EncoderOpenNull);
        }

        Ok(Self {
            encoder,
            yuv: YuvShadowBuffer::i420(width as _, height as _),
        })
    }

    pub fn update_rgba(&mut self, rgba: &[u8], out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        <Self as H264Encoder>::update_rgba(self, rgba, out)
    }

    pub fn get_headers(&mut self, out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        <Self as H264Encoder>::get_headers(self, out)
    }

    fn transfer_nals(&mut self, pp_nal: *mut x264_nal_t, pi_nal: usize, out: &mut BytesMut) {
        let mut size = 0;
        for i in 0..pi_nal {
            let nal = unsafe { *pp_nal.add(i) };
            size += nal.i_payload as usize;
        }

        out.reserve(size);

        for i in 0..pi_nal {
            let nal = unsafe { *pp_nal.add(i) };
            let slice = unsafe { std::slice::from_raw_parts(nal.p_payload, nal.i_payload as usize) };
            out.extend_from_slice(slice);
        }
    }
}

impl H264Encoder for X264FrameBuffer {
    #[trace_time]
    fn update_rgba(&mut self, rgba: &[u8], out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        self.yuv.update(rgba)?;
        let layout = self.yuv.layout();
        let mut pic_in: x264_picture_t = unsafe { std::mem::zeroed() };

        pic_in.img.i_stride[0] = layout.width as _;
        pic_in.img.i_stride[1] = layout.chroma_width as _;
        pic_in.img.i_stride[2] = layout.chroma_width as _;

        let [y, u, v] = self.yuv.yuv_slices_mut();

        pic_in.img.plane[0] = y.as_ptr() as _;
        pic_in.img.plane[1] = u.as_ptr() as _;
        pic_in.img.plane[2] = v.as_ptr() as _;

        pic_in.img.i_csp = X264_CSP_I420 as _;
        pic_in.img.i_plane = 3;
        pic_in.i_type = X264_TYPE_IDR as _;

        let mut pic_out: x264_picture_t = unsafe { std::mem::zeroed() };

        let mut pp_nal: *mut x264_nal_t = ptr::null_mut();
        let mut pi_nal = 0;

        let ret = unsafe { x264_encoder_encode(self.encoder, &mut pp_nal, &mut pi_nal, &mut pic_in, &mut pic_out) };
        if ret < 0 {
            return Err(H264FrameBufferError::X264EncoderEncode(ret));
        }

        self.transfer_nals(pp_nal, pi_nal as _, out);
        Ok(())
    }

    fn get_headers(&mut self, out: &mut BytesMut) -> Result<(), H264FrameBufferError> {
        let mut pp_nal: *mut x264_nal_t = ptr::null_mut();
        let mut pi_nal = 0;

        let ret = unsafe { x264_encoder_headers(self.encoder, &mut pp_nal, &mut pi_nal) };

        if ret < 0 {
            return Err(H264FrameBufferError::X264EncoderHeaders(ret));
        }

        self.transfer_nals(pp_nal, pi_nal as _, out);
        Ok(())
    }
}

impl Drop for X264FrameBuffer {
    fn drop(&mut self) {
        if !self.encoder.is_null() {
            unsafe { x264_encoder_close(self.encoder) };
        }
    }
}
