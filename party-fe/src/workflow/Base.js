import Layer1 from '../tea/layer1';
import utils from '../tea/utils';
import Log from '../shared/utility/Log';
import http from '../tea/http';
import store from '../store';
import request from '../request';

import { _, forge, moment } from 'tearust_utils';
import { hexToString, numberToHex } from 'tearust_layer1';

import '../tea/moment-precise-range';

window._layer1 = require('tearust_layer1');

let _layer1 = null;
let _init = false;
export default class {
  constructor() {
    this.layer1 = _layer1;
    this._log = Log.create(this.defineLog());

    this.gluon = null;
  }

  defineLog() {
    return 'Base';
  }

  async init() {
    const init_loop = (resolve) => {
      if (!this.layer1 || this.layer1.connected !== 2) {
        _.delay(() => {
          init_loop(resolve);
        }, 300);
      }
      else {
        resolve();
      }
    };


    return new Promise(async (resolve) => {
      if (!_init) {
        _init = true;
        await this.initLayer1();
      }
      init_loop(resolve);
    });

  }

  async getAllLayer1Account() {
    const layer1_instance = this.getLayer1Instance();
    if (layer1_instance && layer1_instance.extension) {
      const all_account = await layer1_instance.extension.getAllAccounts();

      return all_account;
    }

    return [];
  }

  async initLayer1() {
    if (!_layer1) {
      _layer1 = new Layer1();

      await _layer1.init();
      await utils.waitLayer1Ready(_layer1);
      this.layer1 = _layer1;
      await this.initEvent();
    }
  }

  async initEvent() {
    const api = this.getLayer1Instance().getApi();
    if (utils.get_env('env') !== 'prod') {
      window.api = api;
    }
    api.rpc.chain.subscribeNewHeads(async (header) => {
      // console.log(`chain is at #${header.number} has hash ${header.hash}`);
      store.commit('set_chain', {
        current_block: header.number,
        current_block_hash: header.hash,
        metadata: this.getLayer1Instance().getMetadata(),
      });
      // const blockInfo = await api.rpc.chain.getBlock(header.hash);

      // const tmp = blockInfo.block.extrinsics;
      // _.each(tmp, (item)=>{
      //   if(item.isSigned){
      //     const rs = {
      //       method: item.method.method,
      //       args: _.map(item.method.args, (v)=>v.toJSON()),
      //       section: item.method.section,
      //       signature: item.signature.toHuman(),
      //       sender: item.signer.toHuman().Id,

      //     };
      //     window.R = rs;
      //     console.log(1, rs)
      //   }
      // })

    });

    const chainInfo = await api.registry.getChainProperties();
    store.commit('set_chain', chainInfo.toHuman());

    // console.log(1, api.errors)
  }

  getLayer1Instance() {
    if (this.layer1) {
      return this.layer1.getLayer1Instance();
    }

    return null;
  }

  async getCurrentBlock(api) {
    if (!api) {
      const layer1_instance = this.getLayer1Instance();
      api = layer1_instance.getApi();
    }
    const block = await api.rpc.chain.getBlock();
    return block.toJSON().block.header.number;
  }

  showQrCodeModal(opts) {
    utils.publish('tea-qrcode-modal', {
      visible: true,
      text: opts.text,
    });
  }
  closeQrCodeModal() {
    utils.publish('tea-qrcode-modal', {
      visible: false,
    });
  }

  blockToDay(block) {
    const hour = 60 * 60 / 6;
    const d = Math.ceil(block / hour);
    if(d < 0) return '0';

    const tmp = moment.utc().preciseDiff(moment.utc().add(d, 'h'), true);
    let rs = '';
    if (tmp.years) {
      rs += tmp.years + 'y';
    }
    if (tmp.months) {
      rs += tmp.months + 'm';
    }
    
    rs += (tmp.days||0) + 'd';

    if(rs === '0d'){
      if(tmp.hours){
        rs = tmp.hours + 'h';
      }
      else if(tmp.minutes){
        rs = tmp.minutes + 'mins'
      }
      else if(tmp.seconds){
        rs = tmp.seconds + 'seconds'
      }
    }

    if(rs === '0d'){
      rs = '0';
    }
    
    return rs;
  }

  encode_b64(str) {
    return forge.util.encode64(str);
  }

  showSelectLayer1Modal() {
    utils.publish('tea-select-layer1-modal', true);
  }

  async getAllDebtByAddress(address){
    const cml_list = await request.layer1_rpc('cml_userCreditList', [
      address
    ]);

    const layer1_instance = this.getLayer1Instance();
    const api = layer1_instance.getApi();

    let total = 0;
    const debt_map = {};

    await Promise.all(_.map(cml_list, async (arr)=>{
      const cml_id = arr[0];
      let debt = parseInt(arr[1], 10);
      // let debt = await api.query.cml.genesisMinerCreditStore(address, cml_id);
      // debt = debt.toJSON();
      if (debt) {
        total += debt;
      }
      _.set(debt_map, cml_id, (debt / layer1_instance.asUnit()))
      return null;
    }));

    total = total / layer1_instance.asUnit();

    
    return {
      total,
      details: debt_map
    };

  }

  async getAllPawnByAddress(address){
    const cml_list = await request.layer1_rpc('cml_userCmlLoanList', [
      address
    ]);

    return cml_list;
  }

  async getAllBalance(address) {
    const layer1_instance = this.getLayer1Instance();
    const api = layer1_instance.getApi();
    let tmp = await api.query.system.account(address);
    // console.log('balance =>', tmp.toJSON().data);
    tmp = tmp.data;

    let reward = await api.query.cml.accountRewards(address);
    reward = reward.toJSON();

    const free = parseInt(tmp.free, 10) / layer1_instance.asUnit();
    const lock = parseInt(tmp.reserved, 10) / layer1_instance.asUnit();
    if (reward) {
      reward = reward / layer1_instance.asUnit();
    }

    

    let usd = await api.query.genesisExchange.uSDStore(address);
    usd = usd.toJSON();
    usd = utils.layer1.balanceToAmount(usd);

    let usd_debt = await api.query.genesisExchange.uSDDebt(address);
    usd_debt = usd_debt.toJSON();
    usd_debt = utils.layer1.balanceToAmount(usd_debt);
    
    return {
      free: Math.floor(free * 10000) / 10000,
      lock: Math.floor(lock * 10000) / 10000,
      reward: reward ? Math.floor(reward * 10000) / 10000 : null,
      usd,
      usd_debt,
    };
  }

  async transferBalance(address, amount) {
    const layer1_account = store.getters.layer1_account;
    if (!layer1_account.address) {
      return false;
    }

    if (!amount || amount === 0) {
      throw 'Invalid transfer balance.';
    }

    if(!address){
      throw 'Invalid receiver\'s address.';
    }

    if(address === layer1_account.address){
      throw 'You cannot send TEA to yourself.';
    }

    const layer1_instance = this.getLayer1Instance();
    const api = layer1_instance.getApi();

    const total = layer1_instance.asUnit() * amount;
    const transfer_tx = api.tx.balances.transfer(address, numberToHex(total));
    await layer1_instance.sendTx(layer1_account.address, transfer_tx);
  }

  async getCoupons(address) {
    const layer1_instance = this.getLayer1Instance();
    const api = layer1_instance.getApi();

    let coupon_investor_A = await api.query.cml.investorCouponStore(address, 'A');
    let coupon_investor_B = await api.query.cml.investorCouponStore(address, 'B');
    let coupon_investor_C = await api.query.cml.investorCouponStore(address, 'C');
    coupon_investor_A = coupon_investor_A.toJSON();
    coupon_investor_B = coupon_investor_B.toJSON();
    coupon_investor_C = coupon_investor_C.toJSON();
    if(coupon_investor_A && coupon_investor_A.amount < 1){
      coupon_investor_A = null;
    }
    if(coupon_investor_B && coupon_investor_B.amount < 1){
      coupon_investor_B = null;
    }
    if(coupon_investor_C && coupon_investor_C.amount < 1){
      coupon_investor_C = null;
    }

    let coupon_team_A = await api.query.cml.teamCouponStore(address, 'A');
    let coupon_team_B = await api.query.cml.teamCouponStore(address, 'B');
    let coupon_team_C = await api.query.cml.teamCouponStore(address, 'C');
    coupon_team_A = coupon_team_A.toJSON();
    coupon_team_B = coupon_team_B.toJSON();
    coupon_team_C = coupon_team_C.toJSON();
    if(coupon_team_A && coupon_team_A.amount < 1){
      coupon_team_A = null;
    }
    if(coupon_team_B && coupon_team_B.amount < 1){
      coupon_team_B = null;
    }
    if(coupon_team_C && coupon_team_C.amount < 1){
      coupon_team_C = null;
    }

    return {
      coupon_investor_A: coupon_investor_A,
      coupon_investor_B: coupon_investor_B,
      coupon_investor_C: coupon_investor_C,
      coupon_team_A: coupon_team_A,
      coupon_team_B: coupon_team_B,
      coupon_team_C: coupon_team_C,
    }
  }

  async refreshCurrentAccount() {

    const layer1_account = store.getters.layer1_account;
    if (!layer1_account.address) {
      return false;
    }

    const layer1_instance = this.getLayer1Instance();

    const api = layer1_instance.getApi();
    const balance = await this.getAllBalance(layer1_account.address);

    const coupons = await this.getCoupons(layer1_account.address);

    const pawn_cml_list = await this.getAllPawnByAddress(layer1_account.address);

    // reset all state
    store.commit('reset_state');

    // let my_auction = await api.query.auction.userAuctionStore(layer1_account.address);
    // my_auction = my_auction.toHuman();
    const cml_list = await this.getCmlListByUser(layer1_account.address);
    const cml_data = await this.getCmlByList(cml_list);

    this._log.i("refresh current layer1_account");
    store.commit('set_account', {
      balance: balance.free,
      lock_balance: balance.lock,
      address: layer1_account.address,
      ori_name: layer1_account.name,
      cml: cml_data,
      reward: balance.reward,
      
      usd: balance.usd,
      usd_debt: balance.usd_debt,

      coupons,
      pawn_cml_list,
    });

    await store.dispatch('init_user');
  }

  async getCmlListByUser(address) {
    const user_cml_list = await request.layer1_rpc('cml_userCmlList', [
      address
    ])

    return user_cml_list;
  }

  async getCmlByList(cml_list) {
    const layer1_instance = this.getLayer1Instance();
    const api = layer1_instance.getApi();

    const current_block = await this.getCurrentBlock(api);

    const unzip_status = (cml) => {
      const status = cml.status;
      let rs = status;
      if (_.isObject(status)) {
        if (_.has(status, 'frozenSeed')) {
          rs = 'FrozenSeed';
        }
        else if (_.has(status, 'staking')) {
          rs = 'Staking';
          cml.staking_cml_id = status.staking.cml_id;
          cml.staking_index = status.staking.staking_index;
        }
        else if (_.has(status, 'tree')) {
          rs = 'Tree';
        }
        else {
          rs = 'FreshSeed';
          cml.fresh_seed_block = status.freshSeed.fresh_seed;
        }
      }
      cml.status = rs;
      return cml;
    };

    const list = await Promise.all(_.map(cml_list, async (cml_id) => {
      let cml = await api.query.cml.cmlStore(cml_id);
      cml = cml.toJSON();

      cml = unzip_status(cml);

      cml.defrost_day = this.blockToDay(cml.intrinsic.generate_defrost_time - current_block);
      let remaining = cml.intrinsic.lifespan;
      if (cml.status !== 'FrozenSeed') {
        remaining = remaining + cml.planted_at - current_block;
      }

      if (remaining < 0) remaining = 0;
      cml.liferemaining = remaining;
      cml.life_day = this.blockToDay(remaining);

      const ttp = await request.layer1_rpc('cml_cmlPerformance', [_.toNumber(cml_id)]);
      const performance = ttp[0]+'/'+ttp[1];

      cml.staking_slot = _.map(cml.staking_slot, (item) => {
        item.category = _.toUpper(item.category);
        return item;
      });
      cml.slot_len = cml.staking_slot.length;

      // status;
      cml.status = ((row) => {
        if (row.status === 'Tree') {
          if (row.staking_slot.length > 0) {
            return 'Mining';
          }
          else {
            return 'Tree'
          }
        }

        return row.status;
      })(cml);

      return {
        ...cml,
        ...cml.intrinsic,
        performance,
        machine_id: hexToString(cml.machine_id),
      };
    }));
// console.log(1, list);
    return list;

  }




}