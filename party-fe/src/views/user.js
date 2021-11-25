
import {_} from 'tearust_utils';
import utils from '../tea/utils';
import bbs from './bbs';
import store from '../store';
import {stringToHex, hexToU8a, stringToU8a} from 'tearust_layer1';

import { signatureVerify } from "@polkadot/util-crypto";

const F = {
  getUserId(address){
    return `profile__${address}`;
  },
  current(address){
    const key = F.getUserId(address);
    const user = utils.cache.get(key);
    if(user){
      return user;
    }

    return null;
  },
  async loginPrepare(layer1_instance, address){
    // thanks for https://github.com/polkadot-js/extension/issues/827
    const data = 'tea-project';

    let sig = await layer1_instance.signWithExtension(address, data);
    sig = utils.uint8array_to_base64(hexToU8a(sig));

    const _axios = bbs.getAxios();
    const rs = await _axios.post('/tapp/loginPrepare', {
      tappId: bbs.getTappId(),
      address,
      data: utils.forge.util.encode64(`<Bytes>${data}</Bytes>`),
      signature: sig,
    });

    const json = JSON.parse(rs);
    const rsaPublicKey = utils.forge.util.decode64(json.rsaPublicKey);

    utils.crypto.set_rsa_publickey(address, rsaPublicKey);

    return true;
  },
  async login(address){
  
    const {key_encrypted, key, rsa_key} = utils.crypto.get_secret(address);

    const _axios = bbs.getAxios();
    const rs = await _axios.post('/tapp/login', {
      tappId: bbs.getTappId(),
      address,
      encryptedAesKey: key_encrypted,
    });
    
    const json = JSON.parse(rs);
    if(json.success){
      // login success
      console.log('login success');

      const user = {
        address,
        isLogin: true,
        rsa: rsa_key,
        aes: key
      };
      // console.log(111, user);
      utils.cache.put(F.getUserId(address), user);

      await store.dispatch('init_user');

      return true;
    }
    console.log('login failed ', rs);
    return false;
  },
  async logout(address){
    // address = '5FzzwcZy6cuBYyMwokDaS7KmMm6xw6H5mwjALjoqBC6pVwLr';

    const _axios = bbs.getAxios();
    const rs = await _axios.post('/tapp/logout', {
      address,
    });
    
    const json = JSON.parse(rs);
    if(json.success){
      utils.cache.remove(F.getUserId(address));
      store.dispatch('init_user');
    }

  
  },
  showLoginModal(self){
    self.$store.commit('modal/open', {
      key: 'login',
      param: {

      },
      cb: async (close)=>{
        self.$root.success('Login success.');
        close();
      }
    })
  }
};

export default F;