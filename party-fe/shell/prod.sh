
echo 'build for prod'
npm run build


echo 'push to a1'
ipfs add -r dist --api /ip4/159.89.149.143/tcp/5001/p2p/12D3KooWGDQ9hb5YqcfFaWEJJVNrs9eii5hgvahTAAbz6iuVZsjH

