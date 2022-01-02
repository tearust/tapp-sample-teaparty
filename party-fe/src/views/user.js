import {_} from 'tearust_utils';
import utils from '../tea/utils';
import bbs from './bbs';
import store from '../store';
import {stringToHex, hexToU8a, stringToU8a} from 'tearust_layer1';


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
  async checkLogin(address, session_key){
    const _axios = bbs.getAxios();
    const rs = await _axios.post('/tapp/checkLogin', {
      tappId: bbs.getTappId(),
      address,
      auth_b64: session_key,
    });
    
    
    console.log(111, rs);
  },
  async loginPrepare(layer1_instance, address){
    // thanks for https://github.com/polkadot-js/extension/issues/827
    const data = 'read_move_withdraw_consume';

    let sig = await layer1_instance.signWithExtension(address, data);
    sig = utils.uint8array_to_base64(hexToU8a(sig));

    try{
      const rs = await bbs.sync_request('login', {
        tappId: bbs.getTappId(),
        address,
        data: utils.forge.util.encode64(`<Bytes>${data}</Bytes>`),
        signature: sig,
      });
  
      const j = rs;
      if(j.ts){
        // query check user via uuid
  
        const r1 = await bbs.sync_request('checkUserAuth', {}, null, 'checkUserAuth', j.uuid);
  
        if(r1.auth_key){
          console.log('login success');
          const user = {
            address,
            isLogin: true,
            session_key: r1.auth_key,
          };
  
          utils.cache.put(F.getUserId(address), user);
          await store.dispatch('init_user');
  
          return true;
        }
      }
    }catch(e){
      console.error(e);
    }
    

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
  async showLoginModal(self){
    self.$root.loading(true);
    const layer1_instance = self.wf.getLayer1Instance();

    if(!self.layer1_account || !self.layer1_account.address){
      self.$root.showError("Invalid user, please select.");
      return;
    }
    const f = await F.loginPrepare(layer1_instance, self.layer1_account.address);
    if(f){
      self.$root.success('Login success.');
    }
    else{
      self.$root.showError("Login failed");
    }

    self.$root.loading(false);
    // self.$store.commit('modal/open', {
    //   key: 'login',
    //   param: {

    //   },
    //   cb: async (close)=>{
    //     self.$root.success('Login success.');
    //     close();
    //   }
    // })
  }
};

export default F;