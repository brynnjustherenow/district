use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use crate::{AppState, db};

const ALLOWED_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];
const MAX_FILE_SIZE: usize = 2 * 1024 * 1024;
const IMAGE_MAGIC: &[(&[u8], &str)] = &[
    (b"\xFF\xD8\xFF", "jpg"),
    (b"\x89PNG\r\n\x1A\n", "png"),
    (b"GIF87a", "gif"),
    (b"GIF89a", "gif"),
    (b"RIFF", "webp"),
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct District {
    pub id: i64,
    pub city: String,
    pub area: String,
    pub description: String,
    pub image: String,
    pub image_file_name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl District {
    pub fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        let image: String = row.get(4)?;
        let file_name = image
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
        Ok(Self {
            id: row.get(0)?,
            city: row.get(1)?,
            area: row.get(2)?,
            description: row.get(3)?,
            image_file_name: file_name,
            image,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDistrict {
    pub city: String,
    pub area: String,
    pub description: String,
    #[serde(default)]
    pub image: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDistrict {
    pub description: String,
    #[serde(default)]
    pub image: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self { code: 0, msg: "ok".into(), data: Some(data) }
    }
    fn error(msg: &str) -> Self {
        Self { code: -1, msg: msg.into(), data: None }
    }
}

pub async fn list_districts(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<District>>>, StatusCode> {
    let conn = state.lock().await;
    match db::list_all(&conn) {
        Ok(list) => Ok(Json(ApiResponse::success(list))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_district(
    State(state): State<AppState>,
    Path((city, area)): Path<(String, String)>,
) -> Result<Json<ApiResponse<District>>, StatusCode> {
    let conn = state.lock().await;
    match db::find_one(&conn, &city, &area) {
        Ok(Some(d)) => Ok(Json(ApiResponse::success(d))),
        Ok(None) => Ok(Json(ApiResponse::error("未找到该区县"))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_district(
    State(state): State<AppState>,
    Json(body): Json<CreateDistrict>,
) -> Result<Json<ApiResponse<i64>>, (StatusCode, Json<ApiResponse<i64>>)> {
    let conn = state.lock().await;
    match db::insert(&conn, &body.city, &body.area, &body.description, &body.image) {
        Ok(id) => Ok(Json(ApiResponse::success(id))),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE") {
                "该区县已存在".to_string()
            } else {
                e.to_string()
            };
            Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(&msg))))
        }
    }
}

pub async fn update_district(
    State(state): State<AppState>,
    Path((city, area)): Path<(String, String)>,
    Json(body): Json<UpdateDistrict>,
) -> Result<Json<ApiResponse<usize>>, (StatusCode, Json<ApiResponse<usize>>)> {
    let conn = state.lock().await;
    match db::update(&conn, &city, &area, &body.description, &body.image) {
        Ok(0) => Err((StatusCode::NOT_FOUND, Json(ApiResponse::error("未找到该区县")))),
        Ok(n) => Ok(Json(ApiResponse::success(n))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&e.to_string())))),
    }
}

pub async fn delete_district(
    State(state): State<AppState>,
    Path((city, area)): Path<(String, String)>,
) -> Result<Json<ApiResponse<usize>>, (StatusCode, Json<ApiResponse<usize>>)> {
    let conn = state.lock().await;
    match db::delete(&conn, &city, &area) {
        Ok(0) => Err((StatusCode::NOT_FOUND, Json(ApiResponse::error("未找到该区县")))),
        Ok(n) => Ok(Json(ApiResponse::success(n))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&e.to_string())))),
    }
}

pub async fn upload_image(
    State(state): State<AppState>,
    Path((city, area)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    {
        let conn = state.lock().await;
        if db::find_one(&conn, &city, &area).unwrap_or(None).is_none() {
            return Err((StatusCode::NOT_FOUND, Json(ApiResponse::error("区县不存在，无法上传图片"))));
        }
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }
        let content_type = field.content_type().unwrap_or("").to_string();
        if !ALLOWED_TYPES.contains(&content_type.as_str()) {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(
                &format!("不支持的文件类型: {}，仅支持 jpg/png/gif/webp", content_type)
            ))));
        }
        let data = field.bytes().await.map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(ApiResponse::error("读取文件失败")))
        })?;
        if data.len() > MAX_FILE_SIZE {
            return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(
                &format!("文件过大，最大允许 {}KB", MAX_FILE_SIZE / 1024)
            ))));
        }
        let ext = match detect_image_ext(&data) {
            Some(e) => e,
            None => return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error(
                "无法识别图片格式，文件可能不是有效的图片"
            )))),
        };
        let filename = format!("{}_{}.{}", sanitize_filename(&city), sanitize_filename(&area), ext);
        let dir = PathBuf::from("uploads");
        fs::create_dir_all(&dir).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&e.to_string())))
        })?;
        let filepath = dir.join(&filename);
        fs::write(&filepath, &data).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&e.to_string())))
        })?;
        let image_url = format!("/uploads/{}", filename);
        let conn = state.lock().await;
        db::update_image(&conn, &city, &area, &image_url).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(&e.to_string())))
        })?;
        return Ok(Json(ApiResponse::success(image_url)));
    }
    Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error("未找到上传文件"))))
}

fn detect_image_ext(data: &[u8]) -> Option<&'static str> {
    for (magic, ext) in IMAGE_MAGIC {
        if data.len() >= magic.len() && &data[..magic.len()] == *magic {
            return Some(ext);
        }
    }
    None
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

pub async fn init_hebei_data(State(state): State<AppState>) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let client = reqwest::Client::new();

    // Step 1: 获取河北省所有市
    let province_url = "https://geo.datav.aliyun.com/areas_v3/bound/130000_full.json";
    let resp = client.get(province_url).send().await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, Json(ApiResponse::error(&format!("请求省份数据失败: {}", e))))
    })?;
    let body: serde_json::Value = resp.json().await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, Json(ApiResponse::error(&format!("解析省份数据失败: {}", e))))
    })?;
    let province_features = body["features"].as_array().ok_or_else(|| {
        (StatusCode::BAD_GATEWAY, Json(ApiResponse::error("省份数据格式异常")))
    })?;

    // 收集所有市: (name, adcode)
    let mut cities: Vec<(String, u64)> = Vec::new();
    for feature in province_features {
        let props = &feature["properties"];
        if props["level"].as_str() == Some("city") {
            let name = props["name"].as_str().unwrap_or("").to_string();
            let adcode = props["adcode"].as_u64().unwrap_or(0);
            if !name.is_empty() && adcode > 0 {
                cities.push((name, adcode));
            }
        }
    }

    // Step 2: 逐市请求区县
    let mut all_districts: Vec<(String, String)> = Vec::new();
    for (city_name, city_adcode) in &cities {
        let city_url = format!(
            "https://geo.datav.aliyun.com/areas_v3/bound/{}_full.json",
            city_adcode
        );
        match client.get(&city_url).send().await {
            Ok(city_resp) => {
                match city_resp.json::<serde_json::Value>().await {
                    Ok(city_body) => {
                        if let Some(city_features) = city_body["features"].as_array() {
                            for feature in city_features {
                                let props = &feature["properties"];
                                if props["level"].as_str() == Some("district") {
                                    let area_name = props["name"].as_str().unwrap_or("").to_string();
                                    if !area_name.is_empty() {
                                        all_districts.push((city_name.clone(), area_name));
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }

    // Step 3: 写入数据库
    let conn = state.lock().await;
    let mut count = 0u32;
    for (city, area) in &all_districts {
        if db::find_one(&conn, city, area).unwrap_or(None).is_none() {
            let _ = db::insert(&conn, city, area, "", "");
            count += 1;
        }
    }

    Ok(Json(ApiResponse::success(format!(
        "已从阿里云API获取河北省行政区划，新增 {} 个区县（共 {} 市 {} 区县）",
        count,
        cities.len(),
        all_districts.len()
    ))))
}
