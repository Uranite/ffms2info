use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;

use ffms2_sys::{
    FFMS_CreateIndexer, FFMS_CreateVideoSource, FFMS_DestroyIndex, FFMS_DestroyVideoSource,
    FFMS_DoIndexing2, FFMS_ErrorInfo, FFMS_GetFirstIndexedTrackOfType, FFMS_GetFrame,
    FFMS_GetVideoProperties, FFMS_Index, FFMS_IndexBelongsToFile, FFMS_Init, FFMS_ReadIndex,
    FFMS_TrackTypeIndexSettings, FFMS_WriteIndex,
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct VidInf {
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub frames: usize,
    pub color_primaries: Option<i32>,
    pub transfer_characteristics: Option<i32>,
    pub matrix_coefficients: Option<i32>,
    pub is_10bit: bool,
    pub color_range: Option<i32>,
    pub chroma_sample_position: Option<i32>,
    pub sar_num: Option<u32>,
    pub sar_den: Option<u32>,
    pub crop_top: u32,
    pub crop_bottom: u32,
    pub crop_left: u32,
    pub crop_right: u32,
    pub is_interlaced: bool,
    pub top_field_first: Option<bool>,
    pub stereo_3d_type: i32,
    pub stereo_3d_flags: i32,
    pub mastering_display: Option<String>,
    pub content_light: Option<String>,
}

pub struct VidIdx {
    pub path: String,
    pub track: i32,
    pub idx_handle: *mut FFMS_Index,
}

impl VidIdx {
    pub fn new(path: &Path) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        unsafe {
            FFMS_Init(0, 0);

            let source = CString::new(path.to_str().ok_or("Invalid path")?)?;
            let mut err = std::mem::zeroed::<FFMS_ErrorInfo>();

            let idx_path = format!("{}.ffindex", path.display());
            let idx_cstr = CString::new(idx_path.as_str())?;

            let mut idx = if std::path::Path::new(&idx_path).exists() {
                FFMS_ReadIndex(idx_cstr.as_ptr(), std::ptr::addr_of_mut!(err))
            } else {
                std::ptr::null_mut()
            };

            if !idx.is_null()
                && FFMS_IndexBelongsToFile(idx, source.as_ptr(), std::ptr::addr_of_mut!(err)) != 0
            {
                FFMS_DestroyIndex(idx);
                idx = std::ptr::null_mut();
            }

            let idx = if idx.is_null() {
                let idxer = FFMS_CreateIndexer(source.as_ptr(), std::ptr::addr_of_mut!(err));
                if idxer.is_null() {
                    return Err("Failed to create idxer".into());
                }

                FFMS_TrackTypeIndexSettings(idxer, 1, 0, 0);

                let idx = FFMS_DoIndexing2(idxer, 0, std::ptr::addr_of_mut!(err));

                if idx.is_null() {
                    return Err("Failed to idx file".into());
                }

                FFMS_WriteIndex(idx_cstr.as_ptr(), idx, std::ptr::addr_of_mut!(err));
                idx
            } else {
                idx
            };

            let track = FFMS_GetFirstIndexedTrackOfType(idx, 0, std::ptr::addr_of_mut!(err));

            Ok(Arc::new(Self {
                path: path.to_str().unwrap().to_string(),
                track,
                idx_handle: idx,
            }))
        }
    }
}

impl Drop for VidIdx {
    fn drop(&mut self) {
        unsafe {
            if !self.idx_handle.is_null() {
                FFMS_DestroyIndex(self.idx_handle);
            }
        }
    }
}

unsafe impl Send for VidIdx {}
unsafe impl Sync for VidIdx {}

fn get_chroma_loc(path: &str, frame_chroma: i32) -> Option<i32> {
    /*
    let ffprobe_out = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=chroma_location",
            "-of",
            "default=noprint_wrappers=1",
            path,
        ])
        .output()
        .ok();

    let ffprobe_val = ffprobe_out.and_then(|out| {
        let text = String::from_utf8_lossy(&out.stdout);
        eprintln!("DEBUG: ffprobe raw output: {:?}", text.trim());
        if text.starts_with("chroma_location=left") {
            Some(1)
        } else if text.starts_with("chroma_location=topleft") {
            Some(3)
        } else {
            None
        }
    });

    eprintln!(
        "DEBUG: ffprobe parsed: {:?}, ffms2 frame_chroma: {}",
        ffprobe_val, frame_chroma
    );
    */
    let ffprobe_val = None;

    let val = ffprobe_val.or_else(|| (frame_chroma != 0).then_some(frame_chroma));

    match val? {
        1 => Some(1),
        3 => Some(2),
        _ => None,
    }
}

pub fn get_vidinf(idx: &Arc<VidIdx>) -> Result<VidInf, Box<dyn std::error::Error>> {
    unsafe {
        let source = CString::new(idx.path.as_str())?;
        let mut err = std::mem::zeroed::<FFMS_ErrorInfo>();

        let video = FFMS_CreateVideoSource(
            source.as_ptr(),
            idx.track,
            idx.idx_handle,
            1,
            1,
            std::ptr::addr_of_mut!(err),
        );

        if video.is_null() {
            return Err("Failed to create vid src".into());
        }

        let props = FFMS_GetVideoProperties(video);
        let frame = FFMS_GetFrame(video, 0, std::ptr::addr_of_mut!(err));

        let matrix_coeff = match if (*frame).ColorSpace == 3 {
            (*props).ColorSpace
        } else {
            (*frame).ColorSpace
        } {
            0 => 2,
            x => x,
        };

        let width = (*frame).EncodedWidth as u32;
        let height = (*frame).EncodedHeight as u32;
        let y_linesize = (*frame).Linesize[0] as usize;
        let is_10bit = y_linesize >= (width as usize) * 2;

        let color_range = match (*frame).ColorRange {
            1 => Some(0),
            2 => Some(1),
            _ => None,
        };

        let chroma_sample_position = get_chroma_loc(&idx.path, (*frame).ChromaLocation);

        let mastering_display = if (*props).HasMasteringDisplayPrimaries != 0
            && (*props).HasMasteringDisplayLuminance != 0
        {
            Some(format!(
                "G({:.4},{:.4})B({:.4},{:.4})R({:.4},{:.4})WP({:.4},{:.4})L({:.4},{:.4})",
                (*props).MasteringDisplayPrimariesX[1],
                (*props).MasteringDisplayPrimariesY[1],
                (*props).MasteringDisplayPrimariesX[2],
                (*props).MasteringDisplayPrimariesY[2],
                (*props).MasteringDisplayPrimariesX[0],
                (*props).MasteringDisplayPrimariesY[0],
                (*props).MasteringDisplayWhitePointX,
                (*props).MasteringDisplayWhitePointY,
                (*props).MasteringDisplayMaxLuminance,
                (*props).MasteringDisplayMinLuminance
            ))
        } else {
            None
        };

        let content_light = if (*props).HasContentLightLevel != 0 {
            Some(format!(
                "{},{}",
                (*props).ContentLightLevelMax,
                (*props).ContentLightLevelAverage
            ))
        } else {
            None
        };

        let sar_num = if (*props).SARNum > 0 {
            Some((*props).SARNum as u32)
        } else {
            None
        };
        let sar_den = if (*props).SARDen > 0 {
            Some((*props).SARDen as u32)
        } else {
            None
        };

        let crop_top = (*props).CropTop as u32;
        let crop_bottom = (*props).CropBottom as u32;
        let crop_left = (*props).CropLeft as u32;
        let crop_right = (*props).CropRight as u32;

        let is_interlaced = (*frame).InterlacedFrame != 0;
        let top_field_first = if is_interlaced {
            Some((*frame).TopFieldFirst != 0)
        } else {
            None
        };

        let stereo_3d_type = (*props).Stereo3DType;
        let stereo_3d_flags = (*props).Stereo3DFlags;

        let inf = VidInf {
            width,
            height,
            fps_num: (*props).FPSNumerator as u32,
            fps_den: (*props).FPSDenominator as u32,
            frames: (*props).NumFrames as usize,
            color_primaries: Some((*frame).ColorPrimaries),
            transfer_characteristics: Some((*frame).TransferCharateristics),
            matrix_coefficients: Some(matrix_coeff),
            is_10bit,
            color_range,
            chroma_sample_position,
            sar_num,
            sar_den,
            crop_top,
            crop_bottom,
            crop_left,
            crop_right,
            is_interlaced,
            top_field_first,
            stereo_3d_type,
            stereo_3d_flags,
            mastering_display,
            content_light,
        };

        FFMS_DestroyVideoSource(video);

        Ok(inf)
    }
}
