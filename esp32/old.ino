#include "WiFi.h"
#include "network_secret.h"
#include <OneWire.h>
#include <DallasTemperature.h>

// WiFi network name and password:
const char *networkName = NETWORK_NAME;
const char *networkPswd = NETWORK_PASSWORD;

// Internet domain to request from:
const char *hostDomain = "example.com";
const int hostPort = 80;

const int BUTTON_PIN = 0;
const int LED_PIN = 5;

bool try_onewire(int i)
{
  Serial.print("trying ");
  Serial.println(i);
  OneWire oneWire(i);
  DallasTemperature sensors(&oneWire);
  sensors.begin();
  sensors.begin();
  for (int j = 0; j < 10; j++)
  {
    int num_devices = sensors.getDeviceCount();
    sensors.requestTemperatures();
    float temperatureC = sensors.getTempCByIndex(0);
    if (num_devices > 0 || temperatureC > -126.99)
    {
      Serial.print(i);
      Serial.println(" works!");
      return true;
    }
    delay(500);
  }
  Serial.print(i);
  Serial.println(" doesn't work");
  return false;
}

void setup()
{
  // Initilize hardware:
  Serial.begin(115200);
  pinMode(BUTTON_PIN, INPUT_PULLUP);
  pinMode(LED_PIN, OUTPUT);

  // Connect to the WiFi network (see function below loop)
  connectToWiFi(networkName, networkPswd);

  // GPIO where the DS18B20 is connected to
  try_onewire(17);
  try_onewire(2);

  digitalWrite(LED_PIN, LOW); // LED off
  Serial.print("Press button 0 to connect to ");
  Serial.println(hostDomain);
}

void loop()
{
  if (digitalRead(BUTTON_PIN) == LOW)
  { // Check if button has been pressed
    while (digitalRead(BUTTON_PIN) == LOW)
      ; // Wait for button to be released

    digitalWrite(LED_PIN, HIGH); // Turn on LED
    //requestURL(hostDomain, hostPort); // Connect to server
    try_onewire(2);
    try_onewire(17);
    digitalWrite(LED_PIN, LOW); // Turn off LED
  }
}

void connectToWiFi(const char *ssid, const char *pwd)
{
  int ledState = 0;

  printLine();
  Serial.println("Connecting to WiFi network: " + String(ssid));

  WiFi.begin(ssid, pwd);

  while (WiFi.status() != WL_CONNECTED)
  {
    // Blink LED while we're connecting:
    digitalWrite(LED_PIN, ledState);
    ledState = (ledState + 1) % 2; // Flip ledState
    delay(500);
    Serial.print(".");
  }

  Serial.println();
  Serial.println("WiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
}

void requestURL(const char *host, uint8_t port)
{
  printLine();
  Serial.println("Connecting to domain: " + String(host));

  // Use WiFiClient class to create TCP connections
  WiFiClient client;
  if (!client.connect(host, port))
  {
    Serial.println("connection failed");
    return;
  }
  Serial.println("Connected!");
  printLine();

  // This will send the request to the server
  client.print((String) "GET / HTTP/1.1\r\n" +
               "Host: " + String(host) + "\r\n" +
               "Connection: close\r\n\r\n");
  unsigned long timeout = millis();
  while (client.available() == 0)
  {
    if (millis() - timeout > 5000)
    {
      Serial.println(">>> Client Timeout !");
      client.stop();
      return;
    }
  }

  // Read all the lines of the reply from server and print them to Serial
  while (client.available())
  {
    String line = client.readStringUntil('\r');
    Serial.print(line);
  }

  Serial.println();
  Serial.println("closing connection");
  client.stop();
}

void printLine()
{
  Serial.println();
  for (int i = 0; i < 30; i++)
  {
    Serial.print("-");
  }
  Serial.println();
}
