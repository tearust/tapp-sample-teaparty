<template>

  <el-dialog
    :title="title"
    :visible="visible"
    width="70%"
    :close-on-click-modal="false"
    custom-class="tea-modal"
    :destroy-on-close="true"
    @opened="openHandler()"
    @close="close()"
  >

    <i v-if="!param || loading" class="el-icon-loading" style="display: block; width: 40px; height: 40px;font-size: 40px; margin: 0 auto;"></i>

    <p v-if="!loading && param.text" class="c-info" :inner-html.prop="param.text"></p>
    
    <el-form v-if="!loading" ref="tx_form" :model="form" :rules="rules" :label-width="(param.label_width||150)+'px'">
      <el-form-item v-for="(item) in args" :key="item.name" :label="labels[item.name]" :prop="item.name" :class="(props[item.name] && props[item.name].class) || ''">
        <el-input v-if="types[item.name]==='Input'" :disabled="props[item.name].disabled||false" v-model="form[item.name]"></el-input>

        <el-select v-if="types[item.name]==='select'" v-model="form[item.name]" placeholder="Please select.">
          <el-option
            v-for="item in props[item.name].options || []"
            :key="item.key || item.id"
            :label="item.label || item.id"
            :value="item.value || item.id"
          >
          </el-option>
        </el-select>
        <el-input-number v-if="types[item.name]==='number'" v-model="form[item.name]" :min="props[item.name].min || 0" :max="props[item.name].max || 50000" :step="props[item.name].step || 1"></el-input-number>

        <TeaIconButton style="margin-left: 10px;" icon_style="font-size:18px;" v-if="props[item.name].tip" :tip="props[item.name].tip" icon="questionmark" />
      </el-form-item>
      
    </el-form>

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="close()">Close</el-button>
      <el-button size="small" :disabled="loading" type="primary" @click="confrim()">
        {{param.confirm_text || 'Confirm'}}
      </el-button>
    </span>

  </el-dialog>


</template>
<script>
import { mapState, mapGetters } from 'vuex';
import store from '../../store/index';
import utils from '../../tea/utils';
import Base from '../../workflow/Base';
import {_} from 'tearust_utils';
import TeaIconButton from '../../components/TeaIconButton';

export default {
  components: {
    TeaIconButton,
  },
  data(){
    return {
      form: null,
      labels: null,
      rules: null,
      types: null,
      args: null,

      loading: true,
    };
  },
  computed: {
    // ...mapGetters([
    //   'layer1_account'
    // ]),
    ...mapState('modal', {
      visible:state => store.state.modal.common_tx.visible,
      title: state => store.state.modal.common_tx.title,
      param: state => store.state.modal.common_tx.param,
    })
  },

  methods: {
    async openHandler(){
      this.wf = new Base();
      await this.wf.init();
      await utils.waitLayer1Ready(this.wf.layer1);
      this.layer1_instance = this.wf.getLayer1Instance();
      this.api = this.layer1_instance.getApi();

      // console.log(1, this.layer1_instance.getMetadata());
      let tx = this.layer1_instance.getMetadata().tx;
      tx = _.get(tx, this.param.pallet+'.'+this.param.tx, null);
      if(!tx){
        throw 'Invalid Tx for common_tx_modal.';
      }
      tx = tx.toJSON();
      tx.props = this.param.props || {};

      const open_cb = utils.mem.get('common_tx--open_cb');
      if(open_cb){
        await open_cb(this.param);
      }

      this.initFormByTx(tx);
      
      this.loading = false;
    },
    initFormByTx(tx){
      const args = tx.args;

      const form = {};
      const labels = {};
      const rules = {};
      const types = {};
      const props = {};

      _.each(args, (item)=>{
        const n = item.name;
        labels[n] = utils.form.nameToLabel(n);
        form[n] = '';

        let type = item.type;
        
        types[n] = 'Input';
        props[n] = {};
        if(tx.props && tx.props[n]){
          props[n] = tx.props[n];
          types[n] = props[n].type;

          if(props[n].default){
            form[n] = props[n].default;
          }

          if(props[n].label){
            labels[n] = props[n].label;
          }
        }

        if(_.startsWith(type, 'Option')){
          type = type.replace('Option<', '').replace('>', '');
          rules[n] = [];
        }
        else{
          rules[n] = [{required: true, message: `${labels[n]} is required.`}];
          if(props[n].rules){
            rules[n] = _.concat(props[n].rules, rules[n]);
          }
        }

        
        
      });

      this.form = form;
      this.labels = labels;
      this.types = types;
      this.rules = rules;
      this.args = args;
      this.props = props;

      
    },

    close(){
      this.$store.commit('modal/close', 'common_tx');
      _.delay(()=>{
        this.loading = true;
      }, 500)
    },
    async confrim(){
      const ref = this.$refs['tx_form'];
      await ref.validate();

      const cb = utils.mem.get('common_tx');
      if(cb){
        const form = this.form;
        await cb(form, ()=>{
          this.close();
        });
      }

    }
  }
}
</script>