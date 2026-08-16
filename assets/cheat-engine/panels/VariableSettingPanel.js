import {TRANSLATE_SETTINGS, TRANSLATOR} from '../js/TranslateHelper.js'

export default {
    name: 'VariableSettingPanel',

    template: `
<v-card flat class="ma-0 pa-0">
    <v-data-table
        v-if="tableHeaders"
        denses
        :headers="tableHeaders"
        :items="filteredTableItems"
        :search="search"
        :custom-filter="tableItemFilter"
        :items-per-page="5">
        <template v-slot:top>
            <v-text-field
                label="Search..."
                solo
                background-color="grey darken-3"
                v-model="search"
                dense
                hide-details
                @keydown.self.stop
                @focus="$event.target.select()">
            </v-text-field>
            <v-row
                class="ma-0 pa-0">
                <v-col
                    cols="12"
                    md="12">
                    <v-checkbox
                        v-model="excludeNameless"
                        dense
                        hide-details
                        label="Hide Nameless Items">
                    
                    </v-checkbox>
                </v-col>
            </v-row>
        </template>
        <template
            v-slot:item.value="{ item }">
            <v-text-field
                background-color="grey darken-3"
                class="d-inline-flex"
                height="10"
                style="width: 60px;"
                hide-details
                solo
                v-model="item.value"
                label="Value"
                dense
                @keydown.self.stop
                @change="onItemChange(item)"
                @focus="$event.target.select()">
            </v-text-field>
        </template>
    </v-data-table>
    
    <v-tooltip
        bottom>
        <span>Reload from game data</span>
        <template v-slot:activator="{ on, attrs }">
            <v-btn
                style="top: 0px; right: 0px;"
                color="pink"
                dark
                small
                absolute
                top
                right
                fab
                v-bind="attrs"
                v-on="on"
                @click="initializeVariables">
                <v-icon>mdi-refresh</v-icon>
            </v-btn>
        </template>
    </v-tooltip>
</v-card>
    `,

    data () {
        return {
            search: '',
            excludeNameless: false,

            variableNames: [],

            tableHeaders: [
                {
                    text: 'Name',
                    value: 'name'
                },
                {
                    text: 'Value',
                    value: 'value'
                }
            ],
            tableItems: []
        }
    },

    created () {
        this.initializeVariables()
    },

    computed: {
        filteredTableItems () {
            return this.tableItems.filter(item => {
                if (this.excludeNameless && !item.name) {
                    return false
                }

                return true
            })
        }
    },

    methods: {
        async initializeVariables () {
            this.variableNames = await this.getVariableNames()

            this.tableItems = this.variableNames.map((varName, idx) => {
                return {
                    id: idx,
                    name: varName,
                    value: $gameVariables.value(idx)
                }
            })
        },

        async getVariableNames () {
            const rawVariableNames = $dataSystem.variables.slice()

            if (TRANSLATE_SETTINGS.isVariableTranslateEnabled()) {
                return await TRANSLATOR.translateBulk(rawVariableNames)
            }

            return rawVariableNames
        },

        onItemChange (item) {
            if (item.displayValue === undefined || item.displayValue === null) return;

            let rawInput = String(item.displayValue).trim();
            let convertedValue = rawInput;

            if (rawInput === '') {
                $gameVariables.setValue(item.id, '');
                return;
            }

            // 진짜 JSON 배열/오브젝트 기호나 불리언일 때만 파싱 처리
            if (/^\[/.test(rawInput) || /^\{/.test(rawInput) || rawInput === 'true' || rawInput === 'false') {
                try {
                    convertedValue = JSON.parse(rawInput);
                } catch (e) {
                    convertedValue = rawInput;
                }
            } 
            // 전체 형태가 숫자인 경우 숫자로 안전하게 변환
            else if (!isNaN(rawInput)) {
                convertedValue = Number(rawInput);
            }

            // modify value
            $gameVariables.setValue(item.id, convertedValue)

            // refresh
            const latestValue = $gameVariables.value(item.id);
            item.value = latestValue;
            item.displayValue = typeof latestValue === 'object' ? JSON.stringify(latestValue) : String(latestValue);
        },

        tableItemFilter (value, search, item) {
            if (search === null || search.trim() === '') {
                return true
            }

            search = search.toLowerCase()

            return item.name.toLowerCase().contains(search) || String(item.value).toLowerCase().contains(search)
        }
    }
}
