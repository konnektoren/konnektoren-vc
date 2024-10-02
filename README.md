# konnektoren-vc
Verifiable Credentials for Konnektoren

## Running with Docker Compose

To run this project using Docker Compose, follow these steps:

1. Ensure you have Docker and Docker Compose installed on your system.

2. Clone this repository:
   ```
   git clone https://github.com/konnektoren/konnektoren-vc.git
   cd konnektoren-vc
   ```

3. Create a `.env` file in the project root and add the necessary environment variables:
   ```
   PRIVATE_KEY=issuer_private_key
   ISSUER_URL=https://vc.konnektoren.help
   DOMAIN=vc.konnektoren.help
   LOG_LEVEL=debug
   ```

4. To run the server without Cloudflare Tunnel:
   ```
   docker-compose up -d server
   ```

5. To run with Cloudflare Tunnel (make sure to add CF_TUNNEL_TOKEN to your .env file):
   ```
   docker-compose --profile cloudflare up -d
   ```

6. To stop the containers:
   ```
   docker-compose down
   ```

Note: Make sure to replace the placeholder values in the .env file with your actual configuration.
