// Minimal UDT4 stream server for PureRust ↔ CppUdt4 interop.
// usage: stream_server <port> <outfile> [expect_bytes]
// Accepts one client, receives until peer closes or expect_bytes reached, writes outfile, exits.

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
  if (argc < 3 || argc > 4 || 0 == atoi(argv[1])) {
    cerr << "usage: stream_server <port> <outfile> [expect_bytes]" << endl;
    return 2;
  }
  const char* port = argv[1];
  const char* outfile = argv[2];
  int64_t expect = (argc == 4) ? atoll(argv[3]) : -1;

  UDT::startup();

  addrinfo hints;
  addrinfo* res;
  memset(&hints, 0, sizeof(hints));
  hints.ai_flags = AI_PASSIVE;
  hints.ai_family = AF_INET;
  hints.ai_socktype = SOCK_STREAM;

  if (0 != getaddrinfo(NULL, port, &hints, &res)) {
    cerr << "bad port" << endl;
    return 1;
  }

  UDTSOCKET serv = UDT::socket(res->ai_family, res->ai_socktype, res->ai_protocol);
  if (UDT::ERROR == UDT::bind(serv, res->ai_addr, res->ai_addrlen)) {
    cerr << "bind: " << UDT::getlasterror().getErrorMessage() << endl;
    return 1;
  }
  freeaddrinfo(res);

  if (UDT::ERROR == UDT::listen(serv, 1)) {
    cerr << "listen: " << UDT::getlasterror().getErrorMessage() << endl;
    return 1;
  }
  cout << "stream_server ready port=" << port << endl;
  cout.flush();

  sockaddr_storage clientaddr;
  int addrlen = sizeof(clientaddr);
  UDTSOCKET recver = UDT::accept(serv, (sockaddr*)&clientaddr, &addrlen);
  if (UDT::INVALID_SOCK == recver) {
    cerr << "accept: " << UDT::getlasterror().getErrorMessage() << endl;
    return 1;
  }
  cout << "stream_server accepted" << endl;
  cout.flush();

  ofstream ofs(outfile, ios::out | ios::binary | ios::trunc);
  char buf[65536];
  int64_t total = 0;
  while (expect < 0 || total < expect) {
    int n = UDT::recv(recver, buf, sizeof(buf), 0);
    if (UDT::ERROR == n) {
      // peer closed / error ends the transfer
      break;
    }
    if (n == 0) {
      break;
    }
    ofs.write(buf, n);
    total += n;
  }
  ofs.close();
  UDT::close(recver);
  UDT::close(serv);
  UDT::cleanup();
  cout << "stream_server received " << total << " bytes" << endl;
  return 0;
}
