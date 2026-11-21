// Minimal UDT4 stream client for PureRust ↔ CppUdt4 interop.
// usage: stream_client <server_ip> <port> <infile>
// Connects, sends entire infile, closes.

#ifndef WIN32
#include <arpa/inet.h>
#include <netdb.h>
#include <unistd.h>
#else
#include <winsock2.h>
#include <ws2tcpip.h>
#endif
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <udt.h>

using namespace std;

int main(int argc, char* argv[]) {
  if (argc != 4 || 0 == atoi(argv[2])) {
    cerr << "usage: stream_client <server_ip> <port> <infile>" << endl;
    return 2;
  }

  UDT::startup();

  addrinfo hints;
  addrinfo* peer;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_INET;
  hints.ai_socktype = SOCK_STREAM;

  if (0 != getaddrinfo(argv[1], argv[2], &hints, &peer)) {
    cerr << "bad address" << endl;
    return 1;
  }

  UDTSOCKET client = UDT::socket(peer->ai_family, peer->ai_socktype, peer->ai_protocol);
  if (UDT::ERROR == UDT::connect(client, peer->ai_addr, peer->ai_addrlen)) {
    cerr << "connect: " << UDT::getlasterror().getErrorMessage() << endl;
    freeaddrinfo(peer);
    return 1;
  }
  freeaddrinfo(peer);
  cout << "stream_client connected" << endl;
  cout.flush();

  ifstream ifs(argv[3], ios::in | ios::binary);
  if (!ifs) {
    cerr << "cannot open infile" << endl;
    return 1;
  }
  char buf[65536];
  int64_t total = 0;
  while (ifs) {
    ifs.read(buf, sizeof(buf));
    streamsize got = ifs.gcount();
    if (got <= 0) {
      break;
    }
    int off = 0;
    while (off < got) {
      int n = UDT::send(client, buf + off, (int)got - off, 0);
      if (UDT::ERROR == n) {
        cerr << "send: " << UDT::getlasterror().getErrorMessage() << endl;
        return 1;
      }
      off += n;
      total += n;
    }
  }
  ifs.close();
  UDT::close(client);
  UDT::cleanup();
  cout << "stream_client sent " << total << " bytes" << endl;
  return 0;
}
