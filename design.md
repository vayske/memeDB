# MemeDB design documentation

## 1. Database Schema
- `Images`
  - `id` - `INTEGER` - key
  - `filename` - `TEXT` - Use hash to distinguish each image
  - `ext` - `TEXT`
  ```
  CREATE TABLE IF NOT EXISTS images (
      id          INTEGER PRIMARY KEY AUTOINCREMENT,
      filename    TEXT UNIQUE NOT NULL
      ext    TEXT UNIQUE NOT NULL
  );
  ```
- `Tags`
  - `id` - `INTEGER` - key
  - `name` - `TEXT` - tag name
  ```
  CREATE TABLE IF NOT EXISTS tags (
      id          INTEGER PRIMARY KEY AUTOINCREMENT,
      name        TEXT UNIQUE NOT NULL
  );
  ```
- `Image_Tags`
  - `image_id` - `INTEGER` - `Image` id
  - `tag_id` - `INTEGER` - `Tag` id
  ```
  CREATE TABLE IF NOT EXISTS image_tags (
      image_id    INTEGER NOT NULL,
      tag_id      INTEGER NOT NULL,
      PRIMARY KEY (image_id, tag_id),
      FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
      FOREIGN KEY (tag_id)   REFERENCES tags(id)   ON DELETE CASCADE
  );
  ```

## 2. API Interface
- `POST /api/upload`
  - **Request**: `multipart/form-data`
    - `file`: binary
  - **Response**:
    ```
    JSON
    {
      "status": 200,
      "data": {
        "id": 1,
        "url": "http://localhost:1234/images/filename.jpg"
      }
    }
  ```
- `POST /api/images/1/tags`
  - **Request**: `application/json`
  ```
  JSON
  {
    "tags": ["cat", "meme"]
  }
  ```
  - **Response**
  ```
  JSON
  {
    "status"： 200,
    "msg": "Tags added",
    "data": {
      "image_id": 1,
      "tags": ["cat", "meme"]
    }
  }
  ```
- `GET /api/search?tags=cat,happy`
  ```
  JSON
  {
    "status": 200,
    "data": [
      {
        "id": 1,
        "url": "http://localhost:1234/images/filename.jpg",
      },
      ...
    ]
  }
  ```
## 3. Core logic

### Upload
1. Frontend uploads file through api call
2. Backend uses `SHA-256` to generate a filename
3. Backend searches database for identical filename
    - if filename not exists -> save file to disk, insert DB, return JSON
    - if exists, do not save and return existing image data

### Search
1. Frontend calls api with tags
2. Backend searches the database to find all images that match
3. Backend returns a list of images if it finds them
