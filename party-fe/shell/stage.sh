
echo 'build for stage'
npm run testnet

echo 'push to 134.209.69.224'
ipfs add -r dist --api /ip4/159.89.149.143/tcp/5001/p2p/12D3KooWGDQ9hb5YqcfFaWEJJVNrs9eii5hgvahTAAbz6iuVZsjH
