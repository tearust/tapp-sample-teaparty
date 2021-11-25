
import {_, axios, moment} from 'tearust_utils';
import utils from '../tea/utils';
import tapp from '../tea/tapp';
import store from '../store';

const default_channel = utils.urlParam('c') || 'test';
console.log('channel => '+default_channel);
let layer2_url = utils.get_env('layer2_url');

if(!_.includes(['127.0.0.1', 'localhost'], location.hostname)){
  layer2_url = `http://${location.hostname}:8000`;
}

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
  return Promise.reject(error);
});

const F = {
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
  async sendMessage(address, msg, channel=default_channel){
    msg = utils.forge.util.encodeUtf8(msg);
    const encrypted_message = utils.forge.util.encode64(utils.crypto.encode(address, msg));
    console.log(121, utils.crypto.encode(address, msg));
    
    const decode_msg = utils.crypto.decode(address, utils.forge.util.decode64(encrypted_message));
    console.log('decode_msg => '+decode_msg);
    const rs = await _axios.post('/tapp/postMessage', {
      tappId: F.getTappId(),
      address,
      channel: F.getChannel(channel),
      // message: msg
      encryptedMessage: encrypted_message,
    });

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
  }
};

F.consts = {
  channel: default_channel,
};

export default F;