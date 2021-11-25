
echo 'build for prod'
npm run build

echo 'push to alice [64.227.105.212]'
ipfs add -r dist --api /ip4/64.227.105.212/tcp/5001/p2p/12D3KooWBXnrRWGMNkE8to5fnD9Z1j9NhTXSxWHkP7ZM648BgD3E

echo 'push to bob [164.90.159.26]'
ipfs add -r dist --api /ip4/164.90.159.26/tcp/5001/p2p/12D3KooWHqVAepKJobXuRZ5btyDdaNMgS3X1iDaMx6yY7TFHG9sh

