import {_, axios, moment, uuid} from 'tearust_utils';
import utils from '../tea/utils';
import tapp from '../tea/tapp';
import store from '../store';
import { hexToString, numberToHex } from 'tearust_layer1';

const default_channel = utils.urlParam('c') || 'test';
console.log('channel => '+default_channel);
let layer2_url = utils.get_env('layer2_url');

if(!_.includes(['127.0.0.1', 'localhost'], location.hostname)){
  layer2_url = `http://${location.hostname}:8000`;
}

const NPC = '5D2od84fg3GScGR139Li56raDWNQQhzgYbV7QsEJKS4KfTGv';

console.log('layer2 url => '+layer2_url);


//set request base url
const _axios = axios.create({
  baseURL: layer2_url,
});

// set request header 
_axios.interceptors.request.use((config)=>{
  
  return config;
});

// set request response
_axios.interceptors.response.use((res)=>{
  if(res.data){
    if(res.data.data){
      return Promise.resolve(res.data.data);
    }
    else{
      return Promise.resolve(null);
    }
  }
}, (error)=>{
  if(error.response && error.response.status === 503){
    const err = error.response.data.error.replace('Invocation failure: Failed to invoke guest call: Guest call failure: Guest call failed: ', '');
    return Promise.reject(err);
  }
  return Promise.reject(error);
});


let _log = console.log;
const F = {

  setLog(log_fn){
    _log = log_fn;
  },
  log(msg){
    _log(msg);
  },

  getUser(address){
    const user = require('./user').default;
    return user.current(address);
  },
  getChannel(channel){
    // if(channel === 'default'){
    //   channel = 'default_'+store.state.bbs.id;
    // }

    return channel;
  },
  getTappId(){
    let id = store.state.bbs.id;
    if(!id){
      id = utils.get_env('tapp_id');
    }

    return _.toNumber(id);
  },
  async loadMessageList(address, channel=default_channel){
    const rs = await _axios.post('/tapp/loadMessageList', {
      tappId: F.getTappId(),
      channel: F.getChannel(channel),
      address: '',
    });

    if(!rs) return [];
    return F.formatMessageList(JSON.parse(rs));

  },
  async updateTappProfile(address){
    const user = F.getUser(address);
    if(!user || !user.isLogin){
      throw 'not_login';
    }
    // TODO if user is not owner, return;

    const opts = {
      tappId: F.getTappId(),
      address,
      authB64: user.session_key,
      postMessageFee: 100,
    };
    const rs = await sync_request('updateTappProfile', opts);
    console.log('updateTappProfile => ', rs);
    return rs;
  },
  async sendMessage(address, msg, channel=default_channel){
    const user = F.getUser(address);
    if(!user || !user.isLogin){
      throw 'Not login';
    }
    

    msg = utils.forge.util.encodeUtf8(msg);
    const encrypted_message = utils.forge.util.encode64(msg);
    // console.log(121, utils.crypto.encode(address, msg));
    
    // const decode_msg = utils.crypto.decode(address, utils.forge.util.decode64(encrypted_message));
    // console.log('decode_msg => '+decode_msg);

    const opts = {
      tappId: F.getTappId(),
      address,
      channel: F.getChannel(channel),
      // message: msg
      encryptedMessage: encrypted_message,
      authB64: user.session_key,
    };
    const rs = await sync_request('postMessage', opts);

    console.log(11, rs);
    
    return rs;
  },
  async delete_message(address, msg_data, channel=default_channel){
    const {id, sender} = msg_data;
    if(sender !== address){
      throw 'Invalid message owner';
    }
    const rs = await _axios.post('/tapp/deleteMessage', {
      tappId: F.getTappId(),
      msgId: id,
      channel: F.getChannel(channel),
    });

    return rs;
  },
  async extend_message(address, msg_data, channel=default_channel){
    const {id, sender} = msg_data;
    if(sender !== address){
      throw 'Invalid message owner';
    }

    const rs = await _axios.post('/tapp/extendMessage', {
      tappId: F.getTappId(),
      msgId: id,
      channel: F.getChannel(channel),
      time: 6*60*60,
    });

    return rs;
  },

  getAxios(){
    return _axios;
  },

  formatMessageList(list){
    const formatter = 'YYYY-MM-DD HH:mm';
    return _.map(list, (item)=>{
      item.utc = moment(item.utc*1000).format(formatter);
      item.utc_expired = moment(item.utcExpired*1000).format(formatter);
      
      return item;
    });
  },


  async showSetNicknameModal(self, address){
    const nickname = tapp.getNickName(address);

    self.$store.commit('modal/open', {
      key: 'common_form',
      param: {
        title: 'Set nickname',

        props: {
          address: {
            label: 'Account',
            type: 'Input',
            disabled: true,
            default: address,
          },
          nick: {
            label: 'Nickname',
            type: 'Input',
            required: true,
            default: nickname,
          }
        },
      },
      cb: async (form, close)=>{
        const nick = form.nick;
        tapp.setNickName(address, nick);

        close();
        self.$root.success();
      }
    });
  },

  async withdrawFromLayer2(self, succ_cb){

  },

  async topupFromLayer1(self, succ_cb){
    const layer1_instance = self.wf.getLayer1Instance();
    const api = layer1_instance.getApi();

    const tappId = F.getTappId();

    self.$store.commit('modal/open', {
      key: 'common_form',
      param: {
        title: 'Topup',
        text: `You will topup to tapp ${tappId} 100 TEA.`,
        props: {
          
        },
      },
      cb: async (form, close)=>{
        self.$root.loading(true);
        const total = utils.layer1.amountToBalance(100);
        const amt = numberToHex(total);

        const tx = api.tx.bondingCurve.topup(NPC, tappId, amt);
        await layer1_instance.sendTx(self.layer1_account.address, tx);
        await succ_cb()
        self.$root.loading(false);
        close();
        self.$root.success();
        
      }
    });
  },

  async query_balance(param){
    const user = F.getUser(param.address);
    console.log(111, user);
    if(!user || !user.isLogin){
      throw 'not_login';
    }

    param = {
      ...param,
      tappId: F.getTappId(),
      authB64: user.session_key,
    };

    const rs = await sync_request('query_balance', param);
    console.log(1, rs);
    return rs ? utils.layer1.balanceToAmount(rs.balance) : null;
  }
};

const sync_request = async (method, param, message_cb, sp_method='query_result', sp_uuid=null) => {
  message_cb = message_cb || ((msg) => {
    msg && _log(msg);
  });
  const _uuid = sp_uuid || uuid();

  message_cb('start first request...');
  try{
    const step1_rs = await _axios.post('/tapp/'+method, {
      ...param,
      uuid: _uuid,
    });
    message_cb('first step result => '+step1_rs);
  }catch(e){
    if(e === 'not_login'){
      throw e;
    }
    
    message_cb(e);
    message_cb('continue request');
  }
  
  utils.sleep(3000);
  message_cb('start second request...');

  let rs = null;
  let n = 0;
  const loop2 = async ()=>{
    if(n>2){
      return;
    }
    try{
      rs = await _axios.post('/tapp/'+sp_method, {
        uuid: _uuid,
      });

    }catch(e){

      // rs = e.message;
      rs = null;
      await utils.sleep(3000);
      n++;
      
      await loop2();
    }

  };

  await loop2();

  if(rs){
    message_cb(rs);
    return JSON.parse(rs);
  }

  return rs;
};

F.test = {
  get_uuid(){
    return uuid();
  }, 
  async request(_uuid, payload, method){
    const step1_rs = await _axios.post('/tapp/'+method, {
      ...payload,
      uuid: _uuid,
    });
    
    console.log('step 1 result => ', step1_rs);
  },
  async result(_uuid){
    return _axios.post('/tapp/query_result', {
      uuid: _uuid,
    });
  },
};

F.consts = {
  channel: default_channel,
};

F.sync_request = sync_request;
export default F;