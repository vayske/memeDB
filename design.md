# MemeDB design documentation

## Architecture

[Client (Browser)] <--> [Axum Static Serve]
|
v
[Axum API Server] <--> [SQLite Database]
|
v
[Local File Storage (CAS)]

## Core logic

### Upload
1. Frontend uploads file through api call
2. Backend uses `SHA-256` to generate a hash `filename`
3. Backend searches database for identical `filename`
    - if `filename` does not exist, save file to disk, insert DB, return JSON
    - if `filename` exists, do not save and return existing image data

### Search
1. Frontend calls api with tags
2. Backend searches the database to find all images that match
3. Backend returns a list of images if it finds them

## Database Schema
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

## API Interface
- ### Upload an image
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
        "filename": "abc123",
        "ext": "jpg"
      }
    }
    ```
- ### Tag an image
  - `POST /api/images/{image_id}/tags`
  - **Request**: `application/json`
    ```
    JSON
    {
      "tags": ["cat", ... ]
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
- ### List tags of an image
  - `GET /api/search?tags=cat,happy`
  - **Response**
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
