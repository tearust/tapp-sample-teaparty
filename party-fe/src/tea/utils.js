
import http from './http';
import Pubsub from 'pubsub-js';

import * as tearust_utils from 'tearust_utils';
import { 
  hexToString, formatBalance, hexToNumber, hexToBn, numberToHex,
  BN_MILLION, isBn, BN, u8aToHex,

} from 'tearust_layer1';

import './index';

import strings from '../assets/string';

const str = (key) => {
  return _.get(strings, key, key);
};

const { _, uuid, forge } = tearust_utils;

// window.L = require('tearust_layer1');

const consts = {
  CmlType: { A: 'A', B: 'B', C: 'C' },
  DefrostScheduleType: { Investor: 'Investor', Team: 'Team' },
  CurveType: {Linear: 'Linear', SquareRoot: 'SquareRoot'},

};

const _MEM = {};
const mem = {
  set(key, val) {
    _MEM[key] = val;
  },
  get(key) {
    return _.get(_MEM, key, null);
  },
  remove(key) {
    delete _MEM[key];
  }
};

const cache = {
  put(id, data) {
    localStorage.setItem(id, JSON.stringify(data));
  },
  get(id) {
    const d = localStorage.getItem(id);
    try {
      return JSON.parse(d);
    } catch (e) {
      return d;
    }
  },
  remove(id) {
    localStorage.removeItem(id);
  },

};

// TODO move to tearust_layer1 pkgs
const layer1 = {
  formatBalance(value, with_icon=false) {
    let is_negative = false;
    if(_.isNumber(value) && value < 0){
      value = Math.abs(value);
      is_negative = true;
    }

    value = F.toBN(value);
    value = F.bnToBalanceNumber(value);
    value = layer1.roundAmount(value);

    if(is_negative){
      return value * -1;
    }

    if(!with_icon) return value;
    const symbol = '<span style="margin-right: 0;" class="iconfont icon-a-TeaProject-T"></span>'
    return symbol + value;

  },
  amountToBalance(value){
    return _.toNumber(value) * (1000000*1000000);
  },
  balanceToAmount(value){
    return layer1.formatBalance(value);
  },
  roundAmount(value){
    return Math.round(value*10000) / 10000;
  },
  toRealBalance(value){
    value = F.toBN(value);
    value = F.bnToBalanceNumber(value);
    const unit = 1000000*1000000;
    return Math.round(value * unit) / unit;
  }
};


const _secret = {
  id: null,
  key : null,
  iv : null,
  hex : null,

  rsa_key: null,
  key_encrypted: null
};
const crypto = {

  get_secret(address){
    address = address || 'NULL';

    const xk = 'crypto-secret-key__'+address;
    if(_secret.id !== xk){
      const __key = localStorage.getItem(xk);
      const key = __key || forge.random.generateSync(16);
      const iv = key;
      const hex = forge.util.bytesToHex(key);

      localStorage.setItem(xk, key);

      _secret.key = key;
      _secret.iv = iv;
      _secret.hex = hex;
      _secret.id = xk;
    }
    return _secret;
  },

  set_rsa_publickey(address, rsa_key){
    crypto.get_secret(address);
    _secret.rsa_key = rsa_key;

    console.log(222, _secret.key);
    _secret.key_encrypted = crypto.rsaEncodeWithRsaPublickKey(_secret.key, _secret.rsa_key);
  },

  encode(address, buffer_data) {
    const {key, iv} = crypto.get_secret(address);
    const cipher = forge.cipher.createCipher('AES-CBC', key);
    cipher.start({iv: iv});
    cipher.update(forge.util.createBuffer(buffer_data));
    // console.log(111, forge.util.createBuffer(buffer_data))
    cipher.finish();
    const encrypted = cipher.output;

    return encrypted.getBytes();
  },
  decode(address, encryptedBytes) {
    const {key, iv} = crypto.get_secret(address);
    const decipher = forge.cipher.createDecipher('AES-CBC', key);
    decipher.start({iv: iv});
    // const encryptedBytes = forge.util.hexToBytes(hex);
    const length = encryptedBytes.length;

    const chunkSize = 1024 * 64;
    let index = 0;
    let decrypted = '';
    do {
      decrypted += decipher.output.getBytes();
      const buf = forge.util.createBuffer(encryptedBytes.substr(index, chunkSize));
      decipher.update(buf);
      index += chunkSize;
    } while(index < length);
    decipher.finish();
    decrypted += decipher.output.getBytes();
    return decrypted;
  },

  // rsa encode with RSA_PUBLICKEY from step 1
  rsaEncodeWithRsaPublickKey(data, rsa_pub){
    console.log(900, data, rsa_pub);
    const tmp = rsa_pub;
    const pub = forge.pki.publicKeyFromPem(tmp);

    let rs = pub.encrypt(data);

    let xxx = F.stringToU8(rs);
    console.log(903, xxx);
    // console.log(904, F.uint8array_to_base64(xxx));
    // console.log(905, F.stringToU8(rs));
    // return forge.util.encode64(rs);
    return F.uint8array_to_base64(xxx);
  },

  sha256(data) {
    const tmp = forge.sha256.create();
    tmp.update(data);
    return tmp.digest().toHex();
  }
};

const form = {
  nameToLabel(name) {
    return _.map(name.split('_'), (n, i) => {
      if (i > 0) return n;
      return _.capitalize(n);
    }).join(' ');
  }
};

let _http_base_url = '';
const F = {
  cache,
  mem,
  crypto,
  forge,
  layer1,
  consts,
  str,
  form,

  getHttpBaseUrl() {
    if (!_http_base_url) {
      throw 'no http url';

    }

    return _http_base_url;
  },
  setHttpBaseUrl(url) {
    _http_base_url = url;
    http.initBaseUrl();
  },

  convertU8ToString(u8_array) {
    return (_.map(u8_array, (x) => String.fromCharCode(x))).join('');
  },

  uuid() {
    return uuid();
  },

  uint8array_to_arraybuffer(uint8) {
    return uint8.buffer.slice(uint8.byteOffset, uint8.byteOffset + uint8.byteLength);
  },
  uint8array_to_base64(uint8) {
    uint8 = F.convertU8ToString(uint8);
    return forge.util.encode64(uint8);
  },
  stringToU8(str){
    var arr = [];
    for (var i = 0, j = str.length; i < j; ++i) {
      arr.push(str.charCodeAt(i));
    }
  
    var tmpUint8Array = new Uint8Array(arr);
    return tmpUint8Array;
  },
  u8ToString(u8_arr){
    var dataString = "";
    for (var i = 0; i < u8_arr.length; i++) {
      dataString += String.fromCharCode(u8_arr[i]);
    }
  
    return dataString;
  },


  get_env(key) {
    if (key === 'env') {
      return process.env.NODE_ENV;
    }

    const x_key = 'VUE_APP_' + _.toUpper(key);
    return _.get(process.env, x_key, null);
  },


  register: (key, cb) => {
    Pubsub.unsubscribe(key);
    Pubsub.subscribe(key, cb);
  },
  publish: Pubsub.publish,

  async sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time))
  },

  toNumber(n) {
    const tmp = n.toString().replace(/,/g, '');
    return _.toNumber(tmp);
  },

  toBN(val){
    if(isBn(val)) return val;
    if(_.isNumber(val)){
      return hexToBn(numberToHex(val));
    }
    if(_.isString(val)){
      if(_.startsWith(val, '0x')){
        return hexToBn(val);
      }

      return new BN(val);
    }

    throw 'Can not convert to BN => '+val;
  },

  bnToBalanceNumber(bn){
    const value = parseInt(bn.toString(),10)/(1000000*1000000);
    // const value = bn.div(BN_MILLION.mul(BN_MILLION)).toNumber();
    return value;
  },


  async waitLayer1Ready(layer1) {
    while (layer1.connected !== 2) {
      await F.sleep(500);
    }
  },

  async getPriceTable() {
    const key = 'staking_price_table';
    const rs = mem.get(key);
    if (rs) return rs;

    const request = (require('../request')).default;
    const rpc_rs = await request.layer1_rpc('cml_stakingPriceTable', []);
    const fn = (n) => {
      return n / (1000000 * 1000000);
    };
    const price_table = _.map(rpc_rs, (n) => fn(n));
    // console.log(111, price_table);
    mem.set(key, price_table);

    return price_table;
  },
  async getStakingWeightByIndex(index, len) {
    const table = await F.getPriceTable();
    const xt = _.slice(table, 0, len);
    const total = _.sum(xt);

    return (Math.round((table[index] / total) * 100000) / 1000) + '%';
  },

  rpcArrayToString(arr){
    return hexToString(u8aToHex(arr));
  },

  urlToLink(url){
    if(!_.startsWith(url, 'https://') && !_.startsWith(url, 'http://')){
      url = 'http://'+url; 
    }

    return url;
  },

  urlParam(key){
    const l = location.search.replace(/^\?/, '');
    let tmp = l.split('&');
    tmp = _.map(tmp, (arr)=>{
      const t = arr.split('=');
      return {
        key: t[0],
        value: t[1],
      }
    });

    const rs =  _.find(tmp, (x)=>x.key === key);
    return rs ? rs.value : null;
  },


};

window.utils = F;
export default F;