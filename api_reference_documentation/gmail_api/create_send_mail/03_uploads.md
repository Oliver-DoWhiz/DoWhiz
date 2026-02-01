# Uploading Attachments

## Overview

The Gmail API enables file uploads when creating/updating drafts or inserting/sending messages through specific endpoints.

## Upload Options

The API supports three upload methods, specified via the `uploadType` parameter:

### 1. Simple Upload

- **Use case**: Small files (5 MB or less), no metadata needed
- **Method**: POST/PUT to `/upload` URI with `uploadType=media`
- **Headers**: `Content-Type` and `Content-Length`

**Example endpoint:**
```
POST https://www.googleapis.com/upload/gmail/v1/users/userId/messages/send?uploadType=media
```

### 2. Multipart Upload

- **Use case**: Small files with metadata in a single request
- **Method**: POST/PUT to `/upload` URI with `uploadType=multipart`
- **Format**: `multipart/related` content type with two parts (metadata first, media second)

**Example endpoint:**
```
POST https://www.googleapis.com/upload/gmail/v1/users/userId/messages/send?uploadType=multipart
```

### 3. Resumable Upload

- **Use case**: Reliable transfer, especially for larger files and network interruptions
- **Process**: Three-step procedure (initiate, save URI, upload file)
- **Method**: POST/PUT to `/upload` URI with `uploadType=resumable`

## Resumable Upload Process

### Step 1: Initiate Session

```
POST /upload/gmail/v1/users/userId/messages/send?uploadType=resumable
```

**Required headers:**
- `X-Upload-Content-Type`: Media MIME type
- `X-Upload-Content-Length`: File size in bytes
- `Content-Type`: Metadata format (if providing metadata)

### Step 2: Save Session URI

Server responds with `200 OK` and provides a `Location` header containing the session URI with an `upload_id` parameter.

### Step 3: Upload File

```
PUT {session_uri}
```

Send the file data with appropriate `Content-Length` header.

## Chunked Uploads

For resumable uploads, files can be split into chunks:
- **Chunk size**: Must be multiple of 256 KB (except final chunk)
- **Header**: Include `Content-Range: bytes 0-524287/2000000`
- **Response**: Server responds with `308 Resume Incomplete` and `Range` header

## Resuming Interrupted Uploads

**Step 1**: Query status with empty PUT request
```
PUT {session_uri}
Content-Length: 0
Content-Range: bytes */2000000
```

**Step 2**: Extract bytes received from response `Range` header

**Step 3**: Resume upload from that point with remaining data

## Best Practices

- Resume uploads failing due to connection issues or 5xx errors
- Use exponential backoff for 5xx server errors
- Retry other failures with a limit (e.g., 10 retries max)
- Restart entirely on `404 Not Found` or `410 Gone` errors

### Exponential Backoff Strategy

Wait periods follow: (2^n) + random milliseconds where n starts at 0 and increments each retry. Example sequence: 1s, 2s, 4s, 8s, 16s delays. Maximum recommended n=5 (~32 second total delay).

## URI Endpoints

Two endpoint types:
- **Upload URI** (`/upload` prefix): For media data transfer
- **Standard URI**: For metadata operations only

## Supported Media Types & Limits

Consult the API reference for:
- Maximum upload file size per method
- Accepted MIME types (e.g., `message/rfc822` for email)

## API Client Libraries

Support available for: .NET, Java, PHP, Python, Ruby
