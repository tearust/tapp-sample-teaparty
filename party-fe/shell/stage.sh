
echo 'build for stage'
npm run testnet

echo 'push to 134.209.69.224'
ipfs add -r dist --api /ip4/134.209.69.224/tcp/5001/p2p/12D3KooWHQjfo3NRhdbcpn8dVdDrPfrHuTYrzrWmBcgxVy8hmFrJ
