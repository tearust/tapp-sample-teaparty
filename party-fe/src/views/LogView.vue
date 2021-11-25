<template>
  
<div class="tea-page">

  <div style="position:relative;">
    <h4 style="
      font-size: 25px;
      color: #666;
      margin: 10px 0 10px;
    ">Billing information</h4>

    <div style="
      margin:5px 0 10px;
      position: absolute;
      right: 50px;
      top: 36px;
      z-index:1;
    ">
    <!-- <el-button v-if="
      layer1_account && tapp &&
      layer1_account.address===tapp.owner
    " @click="expenseHandler()" type="primary" size="small" style="
      width: 150px;
    ">
      Owner expense
    </el-button> -->
      <el-button v-if="layer1_account.address==='5D2od84fg3GScGR139Li56raDWNQQhzgYbV7QsEJKS4KfTGv'" @click="consumeHandler()" type="primary" size="small" style="
        width: 150px;
      ">
        SUDO Comsume
      </el-button>
    </div>
  </div>
  <el-button size="small" style="top: 40px; z-index:2;" class="tea-refresh-btn" type="primary" plain icon="el-icon-refresh" circle @click="refreshList()"></el-button>

  <el-tabs tab-position="top" style="margin-top: 0;" v-model="tab" @tab-click="clickTab()">
    <el-tab-pane name="consume" label="Consume" :lazy="true">
      <Consume :id="id" :tapp="this.tapp" />
    </el-tab-pane>
    <el-tab-pane name="expense" label="Expense" :lazy="true">
      <Expense :id="id" />
    </el-tab-pane>
    <el-tab-pane name="reward" label="Token reward to miners" :lazy="true">
      <Reward :id="id" />
    </el-tab-pane>
    <el-tab-pane name="other" label="Others" :lazy="true">
      <Others :id="id" />
    </el-tab-pane>
    
  </el-tabs>
</div>

</template>
<script>

import { mapGetters, mapState } from 'vuex';
import Base from '../workflow/Base';
import {_} from 'tearust_utils';
import utils from '../tea/utils';
import request from '../request';

import Consume from './tabs/Consume';
import Expense from './tabs/Expense';
import Others from './tabs/Others';
import Reward from './tabs/Reward';

export default {
  components: {
    Consume,
    Expense,
    Reward,
    Others,
  },
  data(){
    return {
      id: null,
      tapp: null,


      tab: null,
    };
  },
  computed: {
    ...mapGetters([
      'layer1_account'
    ]),
  },
  async mounted(){
    this.$root.loading(true);

    this.wf = new Base();
    await this.wf.init();

    const cid = utils.urlParam('id').replace(/[^0-9]*/g, '');
    this.id = _.toNumber(cid);

console.log('tapp id => ', this.id);

    const layer1_instance = this.wf.getLayer1Instance();
    const api = layer1_instance.getApi();
    
    const tmp = (await api.query.bondingCurve.tAppBondingCurve(this.id)).toJSON();


    // if(_.has(tmp.status, 'pending')){
    //   this.$root.showError('TApp is still pending, can\'t visit.');
    //   return false;
    // }

    const arr = await request.layer1_rpc('bonding_tappDetails', [this.id]);
    // if(arr[0].length < 1){
    //   this.$root.showError('Invalid TApp ID, please check.');
    //   return false;
    // }

    this.tapp = {
      theta: (tmp.buy_curve_theta-tmp.sell_curve_theta)/100,
      id: _.toNumber(arr[1]),
      name: utils.rpcArrayToString(arr[0]),
      token_symbol: utils.rpcArrayToString(arr[2]),
      owner: arr[3],
      detail: utils.rpcArrayToString(arr[4]),
      link: utils.rpcArrayToString(arr[5]),
    };

    
    this.tab = 'consume';
    await this.refreshList();

    this.$root.loading(false);
  },
  methods: {
    async refreshList(){

      this.clickTab();
    },
    async consumeHandler(){
      const layer1_instance = this.wf.getLayer1Instance();
      const api = layer1_instance.getApi();
      this.$store.commit('modal/open', {
        key: 'common_tx', 
        param: {
          title: 'Consume',
          pallet: 'bondingCurve',
          tx: 'consume',
          confirm_text: 'Next',
          text: ``,
          props: {
            tapp_id: {
              label: 'TApp Id',
              disabled: true,
              type: 'Input',
              default: this.id,
            },
            tea_amount: {
              label: 'Amount (TEA)',
              type: 'number',
            }
          },
        },
        cb: async (form, close)=>{
          this.$root.loading(true);
          
          const id = _.toNumber(form.tapp_id);
          const amount = utils.layer1.amountToBalance(form.tea_amount);
          
          let estimate = null;

          try{
            estimate = await request.layer1_rpc('bonding_estimateHowMuchTokenBoughtByGivenTea', [
              id, amount
            ]);
            
            estimate = utils.layer1.balanceToAmount(estimate);
          }catch(e){

            this.$root.showError(JSON.stringify(e));
            this.$root.loading(false);
            return false;
          }

          try{
            await this.$confirm(`You will consume <b>${estimate} TOKEN</b> <br/> Are you sure?`, {
              dangerouslyUseHTMLString: true,
            });
          }catch(e){
            this.$root.loading(false);
            return false;
          }
          

          try{
            const tx = api.tx.bondingCurve.consume(id, utils.toBN(amount), stringToHex(form.note||''));
            await layer1_instance.sendTx(this.layer1_account.address, tx);

            close();

            await this.refreshList();
          }catch(e){
            this.$root.showError(e);
          }
          this.$root.loading(false);
        },
      });
    },

    async expenseHandler(){
      const layer1_instance = this.wf.getLayer1Instance();
      const api = layer1_instance.getApi();

      const tapp_id = this.id;

      this.$root.loading(true);
      try{
        const tx = api.tx.bondingCurve.expense(tapp_id);
        await layer1_instance.sendTx(this.layer1_account.address, tx);

        await this.refreshList();
      }catch(e){
        this.$root.showError(e);
      }
      this.$root.loading(false);

    },

    clickTab(){
      utils.publish('home__'+this.tab);
    }
  }
}
</script>