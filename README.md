# rj45less-server
The rj45less-server simply provide 6 digit number to [rj45less-openwrt](https://github.com/pmnxis/rj45less-openwrt) router.

### How server works
- Get request from the rj45less router.
- The server generate some random generator.
- The server check collission from the database (in this application sqlite)
- If the number is new one, store router information and id to database.

## RJ45Less
RJ45Less follows below goal.

- Just working as `PEAP`, but no RADIUS server.
- Should have back compatibility with older device.
- Should have compatibility with <Espressif Wifi-Mesh>(https://docs.espressif.com/projects/esp-idf/en/release-v4.4/esp32/api-reference/network/esp-wifi-mesh.html).
- Some WiFi TxRx+MCU application working easily.
- Only the devices who have have pre-sahred key can connect to WiFi.
- When deploy router or device user don't need to write long password. But each router should have differnt password.
- The device should connect to the mutual engaged router automatically even each routers have non identical SSID or password.

*TODO: Diagram*
 RJ45Less Wi-Fi eco-system that combined with two necessary components and one optional components.
TxRx+MCU and Wi-Fi router have identical pre-shared `Shared Password Seed` and `Group Name`. Wi-Fi router set the unique SSID and password itself with pre-shared data and unique `Mesh ID`. And the TxRx+MCU get `Mesh ID` then try to connect with the password that generated with the pre-shared datas and `Mesh ID`.
