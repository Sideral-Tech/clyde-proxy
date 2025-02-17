# 🤖 Clyde Proxy

Proxy Clyde from one Discord channel to another.

## 🪧 Demo

![Demo](https://github.com/Sideral-Tech/clyde-proxy/assets/13122796/2fc1f5ac-d2cf-4e48-8667-17462a6fb9f9)

## 🏆 How to

1. 👶 Setup the environment variables in the `.env` file:

   - `DISCORD_TOKEN`: The token of the Discord user that will be used as a self-bot
   - `SELF_BOT_USER_ID`: The ID of the Discord user that will be used as a self-bot
   - `OWNER_ID`: The ID of the Discord user that will be used as the owner of the bot

2. 👾 Run Clyde Proxy:

   ```bash
   cargo run
   ```

3. 📕 Use Clyde Proxy:

   - 📝 To see all commands, mention the user followed by the `help` command:

     ![Help command](https://github.com/Sideral-Tech/clyde-proxy/assets/13122796/30409773-022e-4134-886b-400eb95d18ed)

   - ⚙️ To setup the proxy, mention the user followed by the `proxy` command and the ID of a channel `<channel id>` that Clyde participates in. This ID can also be of a DM channel:

     ![Proxy command](https://github.com/Sideral-Tech/clyde-proxy/assets/13122796/9021bfc7-cd55-4dae-89d1-969b75eae258)

   - 💬 To proxy messages from any channel to Clyde, mention the self-bot or reference a self-bot's message followed by the message you want to send:

     ![Message command](https://github.com/Sideral-Tech/clyde-proxy/assets/13122796/fac852e5-8261-4a8d-956d-57af1245123c)

   - ⏸️ If you want to toggle the proxy on or off, mention the user followed by the `toggle` command:

     ![Toggle command](https://github.com/Sideral-Tech/clyde-proxy/assets/13122796/b14563a6-4128-4ab0-88a7-3797ee12f4f0)

## 💡 Authors

- [@oSumAtrIX](https://osumatrix.me)

## 🪪 License

[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)
