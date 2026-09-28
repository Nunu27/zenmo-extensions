wit_bindgen::generate!({
    world: "test",
    path: "../wit",
});

use serde::{Deserialize, Serialize};
use wasip2::http::{outgoing_handler::OutgoingRequest, types::Fields};

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowseMangaDto {
    pub items: Vec<MangaDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaObjectDto {
    #[serde(rename = "mangaPage")]
    pub manga_page: MangaDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultsDto {
    pub page: i32,
    pub found: i32,
    pub hits: Vec<SearchMangaDto>,
    #[serde(rename = "request_params")]
    pub request_params: RequestParamsDto,
}

impl SearchResultsDto {
    pub fn has_next_page(&self) -> bool {
        self.page * self.request_params.per_page < self.found
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMangaDto {
    pub document: MangaDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestParamsDto {
    #[serde(rename = "per_page")]
    pub per_page: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDto {
    // Common
    pub id: String,
    pub title: String,
    // #[serde(alias = "poster", alias = "image")]
    // pub image_path: Value, // Value is equivalent to JsonElement
    pub large_image: Option<String>,

    // Details
    // pub authors: Option<Value>,
    pub synopsis: Option<String>,
    // #[serde(alias = "genres", alias = "tags")]
    // pub genres: Option<Value>,
    pub released: Option<i64>,
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub manga_type: Option<String>, // 'type' is a reserved keyword in Rust
    // pub views: Option<Value>,
    pub other_names: Option<Vec<String>>,
    pub avg_rating: Option<f32>,
    pub scanlators: Option<Vec<ScanlatorDto>>,

    // Chapters
    pub chapters: Option<Vec<ChapterDto>>,
    pub recommendations: Option<Vec<MangaDto>>,
}

// impl MangaDto {
//     /// Extracts the image path from the generic JSON Value
//     pub fn get_image_path(&self) -> Option<String> {
//         let url = if let Some(ref large) = self.large_image {
//             Some(large.clone())
//         } else if self.image_path.is_string() {
//             self.image_path.as_str().map(String::from)
//         } else if self.image_path.is_object() {
//             self.image_path.get("largeImage")
//                 .or_else(|| self.image_path.get("image"))
//                 .and_then(|v| v.as_str())
//                 .map(String::from)
//         } else {
//             None
//         };

//         url.map(|u| {
//             let u = u.strip_prefix("/").unwrap_or(&u);
//             u.strip_prefix("static/").unwrap_or(u).to_string()
//         })
//     }

//     /// Parses names from the generic JSON Value array
//     pub fn parse_names(&self, element: Option<&Value>) -> Vec<String> {
//         match element {
//             Some(Value::Array(arr)) => arr.iter().filter_map(|item| {
//                 if let Some(s) = item.as_str() {
//                     Some(s.to_string())
//                 } else if let Some(obj) = item.as_object() {
//                     obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
//                 } else {
//                     None
//                 }
//             }).collect(),
//             _ => vec![],
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanlatorDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllChaptersDto {
    pub chapters: Vec<ChapterDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterDto {
    pub id: String,
    pub number: f32,
    pub title: String,
    #[serde(rename = "scanlationMangaId")]
    pub scanlation_manga_id: Option<String>,
    // #[serde(rename = "createdAt")]
    // pub date: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageObjectDto {
    #[serde(rename = "readChapter")]
    pub read_chapter: PageDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageDto {
    pub pages: Vec<PageDataDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageDataDto {
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterData {
    pub genres: Option<Vec<Filter>>,
    pub tags: Option<Vec<Filter>>,
    pub types: Option<Vec<Filter>>,
    pub statuses: Option<Vec<Filter>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub id: String,
    pub name: String,
}

struct Plugin;

impl exports::zenmo::test::plugin_service::Guest for Plugin {
    fn hello_world(data: String) -> String {
        let from_host = zenmo::test::host_service::hello_world(&data);
        format!("Hello from the plugin! The host said: {from_host}")
    }

    fn request() -> String {
        use std::io::Read;

        let headers = Fields::new();
        let req = OutgoingRequest::new(headers);

        let _ = req.set_method(&wasip2::http::types::Method::Get);
        let _ = req.set_scheme(Some(&wasip2::http::types::Scheme::Https));
        let _ = req.set_authority(Some("atsu.moe"));
        let _ = req.set_path_with_query(Some("/api/home2/popular"));

        let res = match wasip2::http::outgoing_handler::handle(req, None) {
            Ok(response) => response,
            Err(error) => return format!("Outgoing request failed: {error:?}"),
        };

        // FutureIncomingResponse is asynchronous; we must subscribe and block until ready
        let pollable = res.subscribe();
        pollable.block();

        let response = match res.get() {
            Some(Ok(Ok(response))) => response,
            Some(Ok(Err(err))) => return format!("HTTP error: {err:?}"),
            Some(Err(())) => return "Response already retrieved".to_string(),
            None => return "Response timed out or not ready".to_string(),
        };

        let status = response.status();
        let body = match response.consume() {
            Ok(body) => body,
            Err(_) => return format!("Status {status}: failed to consume body"),
        };

        let mut stream = match body.stream() {
            Ok(stream) => stream,
            Err(_) => return format!("Status {status}: failed to open body stream"),
        };

        // Preallocate vector if Content-Length header is present
        let mut body_bytes = response
            .headers()
            .get("content-length")
            .first()
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|s| s.parse::<usize>().ok())
            .map(Vec::with_capacity)
            .unwrap_or_default();

        if let Err(err) = stream.read_to_end(&mut body_bytes) {
            return format!("Failed to read stream: {err:?}");
        }

        // Child resources (stream) must be dropped before parent body is dropped
        drop(stream);
        drop(body);

        match serde_json::from_slice::<BrowseMangaDto>(&body_bytes) {
            Ok(browse_manga) => browse_manga
                .items
                .iter()
                .map(|manga| manga.title.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            Err(err) => format!("Failed to deserialize BrowseMangaDto: {err}"),
        }
    }
}

export!(Plugin);
