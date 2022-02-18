import Vuex from 'vuex';
import Vue from 'vue';
import _ from 'lodash';

import Base from '../workflow/Base';

import modal from './modal';
import clog from './clog';
import utils from '../tea/utils';
import request from '../request';
import user from '../views/user';

Vue.use(Vuex);

let wf = null;
const F = {
  async getLayer1() {
    wf = new Base();
    await wf.init();
    return wf.layer1;
  },
  async getWF() {
    if(wf) return wf;
    await F.getLayer1();
    return wf;
  },

  formatAuctionBidData(d){
    if(d.starting_price){
      d.starting_price = utils.toBN(d.starting_price);
    }
    if(d.buy_now_price){
      d.buy_now_price = utils.toBN(d.buy_now_price);
    }
    if(d.price){
      d.price = utils.toBN(d.price);
    }
    return d;
  }
};


const MIN_AUCTION_ID = 1;
const initState = () => {
  return {
    layer1_account: {
      name: null,
      address: null,
      balance: null,
      cml: [],
      reward: null,
      debt: null,
    },

    // address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
    // uuid: '29308409284023805283502845038453095803485308503',
    bind_mobile: null,

    btc_list: [],

    latest_meta: {
      delegator_nonce: null,
      delegator_nonce_hash: null,
      delegator_nonce_rsa: null,
    },


    layer1_asset: {
      dot: []
    },

    auction: {
      last_auction_id: 0,

      auction_list: [],

      my_auction_list: [],
      my_bid_list: [],
    },

    chain: {
      current_block: null,
      current_block_hash: null,
    },

    user: null,

    bbs: {
      id: null,
      channel: null,
      tapp: null,
    },
  }
};

const store = new Vuex.Store({
  modules: {
    modal: modal,
    clog: clog,
  },

  state: initState(),

  getters: {
    layer1_account: (state) => {
      if (state.layer1_account.address) {
        return state.layer1_account;
      }
      const ll = localStorage.getItem('tea-layer1-account');
      if (ll) {

        return JSON.parse(ll);
      }

      return state.layer1_account;
    }
  },

  mutations: {
    set_account(state, account) {
      state.layer1_account = {
        name: account.ori_name,
        address: account.address,
        balance: account.balance,
        lock_balance: account.lock_balance,
        cml: account.cml || [],
        reward: account.reward,
        // debt: account.debt,
        // debt_detail: account.debt_detail,
        usd_debt: account.usd_debt,
        usd: account.usd,
        pawn_cml_list: account.pawn_cml_list,
        ...account.coupons || {},
      };

      localStorage.setItem('tea-layer1-account', JSON.stringify(state.layer1_account));
    },

    set_chain(state, chain_data) {
      state.chain = _.extend(state.chain, chain_data || {});
    },

    set_user(state, user){
      state.user = user;
    },

    reset_state(state) {
      const init_state = initState();
      // Object.keys(init_state).forEach(key => {
      //   state[key] = init_state[key]
      // })
      state.layer1_account = init_state.layer1_account;
      state.auction.my_auction_list = [];
      state.auction.my_bid_list = [];
    },

    set_bbs(state, bbs){
      state.bbs = bbs;
    }

  },

  actions: {
    async set_layer1_asset(store) {
      const layer1_account = store.getters.layer1_account;
      if (!layer1_account) {
        throw 'Invalid layer1 account';
      }

      const layer1 = await F.getLayer1();
      const layer1_instance = layer1.getLayer1Instance();
      // const gluon_pallet = layer1_instance.getGluonPallet();

      // const asset = await gluon_pallet.getAccountAssets(layer1_account.address);

      store.commit('set_layer1_asset', null);
    },

    async init_user(store){
      const layer1_account = store.getters.layer1_account;
      if(!layer1_account){
        throw 'Invalid layer1 account';
      }

      const address = layer1_account.address;
      const me = user.current(address);
console.log('refresh user => ', me);
      if(!me){
        store.commit('set_user', null);
      }
      else{
        store.commit('set_user', me);
      }
      
    }
  }
})

export default store;