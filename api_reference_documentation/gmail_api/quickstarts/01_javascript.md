# JavaScript Quickstart for Gmail API

## Overview

This guide demonstrates how to create a JavaScript web application that interfaces with the Gmail API. The quickstart uses simplified authentication suitable for testing environments.

## Objectives

- Configure your development environment
- Set up the sample application
- Execute and test the sample code

## Prerequisites

Required tools and accounts:
- Node.js and npm installed
- A Google Cloud project configured
- A Google account with Gmail enabled

## Environment Setup

### Enable Gmail API

Access the Google Cloud console and activate the Gmail API through the enablement flow.

### Configure OAuth Consent Screen

If this is your first time using the Cloud project, configure the consent screen:

1. Navigate to Google Auth platform settings in the Cloud console
2. Complete app information with a name for your application
3. Provide a support email for user inquiries
4. Set audience as "Internal" for testing
5. Enter contact information for notifications
6. Review and accept the Google API Services User Data Policy
7. Finalize the configuration

### Create OAuth 2.0 Credentials

Set up credentials for web application authentication:

1. Generate a new OAuth 2.0 Client ID
2. Select "Web application" as the type
3. Register authorized JavaScript origins for browser requests
4. Record the Client ID for later use

### Generate API Key

Obtain an API key from the credentials section and note it for implementation.

## Sample Implementation

Create an `index.html` file containing the complete JavaScript application. The code includes:

- Authorization buttons for user sign-in and sign-out
- Integration with Google's authentication libraries
- Gmail API client initialization
- Label listing functionality

Replace placeholder values (`YOUR_CLIENT_ID` and `YOUR_API_KEY`) with actual credentials.

## Running the Application

1. Install the http-server package via npm
2. Start a local web server on port 8000
3. Navigate to `http://localhost:8000` in your browser
4. Click the authorization button
5. Complete the Google account authentication flow
6. Accept permission requests

The application will then display Gmail labels from your account.

## Next Steps

- Explore additional Gmail API methods in the API reference
- Review authentication troubleshooting guides
- Examine sample implementations for expanded functionality
