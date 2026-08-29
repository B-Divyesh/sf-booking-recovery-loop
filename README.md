# Booking Recovery Loop

Recover a stopped paid booking with recorded email or SMS consent and delivery
receipts. It is for solo coaches, tutors, and consultants.

Try the isolated sample at
`https://booking-recovery-loop.sociobot.in/?demo=1`.

## How booking recovery works

1. Create a practice at `/start` and publish its `/b/<slug>` booking page.
2. A client records email or SMS consent before hosted payment opens.
3. The service schedules a consented recovery message 15 minutes after an unpaid booking.
4. After the payment provider confirms the deposit, the service schedules one session reminder.
5. Delivery receipts appear in the practice queue. A bounced email can use one
   permitted SMS fallback.
6. Export or delete practice data from `/app/settings/data`.

Recovery Loop Practice is $29/month for one practice. Open the official
[Sociobot checkout](https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout)
after setup.

The demo has separate sample storage. It sends no real messages or payments.
Every product promise above is listed with its executable evidence in
[`.factory/claims.json`](.factory/claims.json).

## Run locally

Requirements: Node 22+, npm, and current stable Rust.

```sh
npm ci
npm run build
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:8080`.

## Verify

```sh
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
npm run build
npm run check:size
```

## Deploy

```sh
docker build -f Dockerfile --build-arg BUILD_SHA=local -t booking-recovery-loop .
docker run --rm -p 8080:8080 booking-recovery-loop
```

Set `PORT` when you need a port other than 8080. Use `/health` for a health
check.

## License

[MIT](LICENSE). Fraunces and Atkinson Hyperlegible Next use the SIL Open Font
License; their license texts are in `public/fonts/`.
