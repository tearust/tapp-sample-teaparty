
echo 'build for prod'
npm run build


echo 'push to ipfs bootnode [/ip4/64.227.49.206/tcp/5001/p2p/12D3KooWScg336x2Rzc97ZnHbYAEd592P3DqkYJFZRQneGopjsyT]'
ipfs add -r dist --api /ip4/64.227.49.206/tcp/5001/p2p/12D3KooWScg336x2Rzc97ZnHbYAEd592P3DqkYJFZRQneGopjsyT

