import forge from 'node-forge';
import _ from 'lodash';
import Log from './Log';

const map = {};
const log = Log.create('ED25519');

export default class {
  static keypair = ()=>{
    const seed = forge.random.getBytesSync(32);
    const tmp = forge.ed25519.generateKeyPair({seed});

    _.set(map, tmp.publicKey, tmp.privateKey);

    log.d('pub => ', forge.util.bytesToHex(tmp.publicKey));
    log.d('pri => ', forge.util.bytesToHex(tmp.privateKey));

    return {
      pub: forge.util.bytesToHex(tmp.publicKey),
      pri: forge.util.bytesToHex(tmp.privateKey)
    };
  };

  static sign = (msg, pri_hex)=>{
    const pri = forge.util.hexToBytes(pri_hex);

    let signature = forge.ed25519.sign({
      message: msg,
      encoding: 'utf8',
      privateKey: pri
    });

    signature = forge.util.bytesToHex(signature);
    log.d('signature =>', signature);

    return signature
  }
  // sign_bytes(msg){
  //   let signature = forge.ed25519.sign({
  //     message: msg,
  //     encoding: 'binary',
  //     privateKey: S.privateKey
  //   });

  //   signature = forge.util.bytesToHex(signature);
  //   console.log('signature => %s', signature);

  //   return signature
  // }
}