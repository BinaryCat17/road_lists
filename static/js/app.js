import { createApp, ref, onMounted, computed } from 'vue';

const App = {
  setup() {
    const currentTab = ref('print'); // print, settings, drivers, vehicles, works
    const statusMessage = ref('');
    
    const drivers = ref([]);
    const vehicles = ref([]);
    const works = ref([]);
    const companySettings = ref({ company_name: '', company_address: '', company_inn: '', dispatcher_name: '', mechanic_name: '', medic_name: '' });

    const defaultValues = ref({
      customer: '', loading_point: '', unloading_point: '', cargo: '', trips: '', distance: '', tons: '', arrival_time: '',
      field_object: '', field_area: '', field_norm: '', field_fact: '', field_motohours: ''
    });

    const loadData = async () => {
      try {
        const [dRes, vRes, wRes, sRes, defRes] = await Promise.all([
          fetch('/api/drivers'),
          fetch('/api/vehicles'),
          fetch('/api/works'),
          fetch('/api/settings'),
          fetch('/api/defaults')
        ]);
        drivers.value = await dRes.json();
        vehicles.value = await vRes.json();
        works.value = await wRes.json();
        const s = await sRes.json();
        if (s) companySettings.value = { ...companySettings.value, ...s };
        const d = await defRes.json();
        if (d) {
          defaultValues.value = { ...defaultValues.value, ...d };
          applyDefaults();
        }
      } catch (e) {
        statusMessage.value = 'Ошибка загрузки данных из БД!';
      }
    };

    onMounted(loadData);

    // --- Логика очереди печати ---
    const selectedDriver = ref('');
    const selectedVehicle = ref('');
    const selectedDate = ref(new Date().toLocaleDateString('ru-RU'));
    const tractorMode = ref('cargo'); // 'cargo' | 'field'

    const selectedVehicleObj = computed(() => vehicles.value.find(v => v.id == selectedVehicle.value));
    const isTractorSelected = computed(() => selectedVehicleObj.value?.vehicle_type === 'Трактор');

    const makeTaskFromDefaults = () => ({
      customer: defaultValues.value.customer || '',
      loading_point: defaultValues.value.loading_point || '',
      unloading_point: defaultValues.value.unloading_point || '',
      cargo: defaultValues.value.cargo || '',
      trips: defaultValues.value.trips || '',
      distance: defaultValues.value.distance || '',
      tons: defaultValues.value.tons || '',
      arrival_time: defaultValues.value.arrival_time || ''
    });
    const taskRows = ref([makeTaskFromDefaults()]);
    const addTaskRow = () => {
      if (taskRows.value.length < 3) taskRows.value.push(makeTaskFromDefaults());
    };
    const removeTaskRow = (idx) => {
      if (taskRows.value.length > 1) taskRows.value.splice(idx, 1);
    };

    const makeFieldRowFromDefaults = () => ({
      object: defaultValues.value.field_object || '',
      area: defaultValues.value.field_area || '',
      norm: defaultValues.value.field_norm || '',
      fact: defaultValues.value.field_fact || '',
      motohours: defaultValues.value.field_motohours || ''
    });
    const fieldRows = ref([makeFieldRowFromDefaults()]);
    const addFieldRow = () => {
      if (fieldRows.value.length < 3) fieldRows.value.push(makeFieldRowFromDefaults());
    };
    const removeFieldRow = (idx) => {
      if (fieldRows.value.length > 1) fieldRows.value.splice(idx, 1);
    };

    const applyDefaults = () => {
      taskRows.value = [makeTaskFromDefaults()];
      fieldRows.value = [makeFieldRowFromDefaults()];
    };

    const printQueue = ref([]);

    const addToQueue = () => {
      if (!selectedDriver.value || !selectedVehicle.value) {
        alert('Пожалуйста, выберите водителя и технику!');
        return;
      }
      let tasks = [];
      let mode = null;
      if (isTractorSelected.value && tractorMode.value === 'field') {
        mode = 'field';
        tasks = fieldRows.value.map(r => ({
          customer: '',
          loading_point: r.object,
          unloading_point: '',
          cargo: '',
          trips: r.fact,
          distance: r.area,
          tons: r.norm,
          arrival_time: r.motohours,
        }));
      } else {
        tasks = taskRows.value.map(t => ({ ...t }));
      }
      printQueue.value.push({
        id: Date.now(),
        driver_id: parseInt(selectedDriver.value),
        vehicle_id: parseInt(selectedVehicle.value),
        date: selectedDate.value,
        driver_name: drivers.value.find(d => d.id == selectedDriver.value)?.name,
        vehicle_name: selectedVehicleObj.value?.name,
        tasks: tasks,
        tractor_mode: mode,
      });
    };

    const removeFromQueue = (id) => {
      printQueue.value = printQueue.value.filter(q => q.id !== id);
    };

    const printBatch = async () => {
      if (printQueue.value.length === 0) {
        alert('Очередь пуста!');
        return;
      }
      statusMessage.value = 'Генерация путевых листов...';
      try {
        const response = await fetch('/api/print_batch', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ items: printQueue.value })
        });
        const data = await response.json();
        if (data.success && data.pdf_url) {
          statusMessage.value = 'Готово! Открываю PDF...';
          window.open(data.pdf_url + '?t=' + Date.now(), '_blank');
        } else {
          statusMessage.value = 'Ошибка: ' + data.message;
        }
      } catch (e) {
        statusMessage.value = 'Ошибка связи с сервером!';
      }
    };

    // --- Логика CRUD ---
    const formDriver = ref({ id: null, name: '', driving_license: '', tractor_license: '', snils: '' });
    const formVehicle = ref({ id: null, name: '', license_plate: '', sts: '', vehicle_type: 'Грузовой', category: '' });
    const formWork = ref({ id: null, name: '' });

    const editDriver = (d) => formDriver.value = { ...d, driving_license: d.driving_license || '', tractor_license: d.tractor_license || '', snils: d.snils || '' };
    const editVehicle = (v) => formVehicle.value = { ...v, license_plate: v.license_plate || '', sts: v.sts || '', vehicle_type: v.vehicle_type || 'Грузовой', category: v.category || '' };
    const editWork = (w) => formWork.value = { ...w };

    const clearForm = (type) => {
      if (type === 'driver') formDriver.value = { id: null, name: '', driving_license: '', tractor_license: '', snils: '' };
      if (type === 'vehicle') formVehicle.value = { id: null, name: '', license_plate: '', sts: '', vehicle_type: 'Грузовой', category: '' };
      if (type === 'work') formWork.value = { id: null, name: '' };
    };

    const saveItem = async (type, payload) => {
      const url = `/api/${type}s`;
      const method = payload.id ? 'PUT' : 'POST';
      try {
        await fetch(url, {
          method,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        await loadData();
        clearForm(type);
      } catch (e) {
        alert('Ошибка сохранения!');
      }
    };

    const deleteItem = async (type, id) => {
      if (!confirm('Удалить запись?')) return;
      try {
        await fetch(`/api/${type}s`, {
          method: 'DELETE',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ id })
        });
        await loadData();
      } catch (e) {
        alert('Ошибка удаления!');
      }
    };

    const saveSettings = async () => {
      try {
        await fetch('/api/settings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(companySettings.value)
        });
        statusMessage.value = 'Реквизиты сохранены';
        setTimeout(() => statusMessage.value = '', 2000);
      } catch (e) {
        alert('Ошибка сохранения реквизитов');
      }
    };

    const saveDefaults = async () => {
      try {
        await fetch('/api/defaults', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(defaultValues.value)
        });
        statusMessage.value = 'Значения по умолчанию сохранены';
        setTimeout(() => statusMessage.value = '', 2000);
      } catch (e) {
        alert('Ошибка сохранения значений по умолчанию');
      }
    };

    return {
      currentTab, statusMessage,
      drivers, vehicles, works, companySettings, defaultValues,
      selectedDriver, selectedVehicle, selectedDate,
      tractorMode, isTractorSelected, selectedVehicleObj,
      taskRows, addTaskRow, removeTaskRow,
      fieldRows, addFieldRow, removeFieldRow,
      printQueue, addToQueue, removeFromQueue, printBatch, saveSettings, saveDefaults,
      formDriver, formVehicle, formWork,
      editDriver, editVehicle, editWork, clearForm, saveItem, deleteItem
    };
  },
  template: `
    <div class="max-w-6xl mx-auto p-4 mt-8">
      
      <!-- Навигация -->
      <div class="flex space-x-2 mb-6 border-b border-gray-200 pb-2 overflow-x-auto">
        <button @click="currentTab = 'print'" :class="currentTab === 'print' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Печать листов</button>
        <button @click="currentTab = 'settings'" :class="currentTab === 'settings' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Реквизиты</button>
        <button @click="currentTab = 'defaults'" :class="currentTab === 'defaults' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">По умолчанию</button>
        <button @click="currentTab = 'drivers'" :class="currentTab === 'drivers' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Водители</button>
        <button @click="currentTab = 'vehicles'" :class="currentTab === 'vehicles' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Техника</button>
        <button @click="currentTab = 'works'" :class="currentTab === 'works' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Виды работ</button>
      </div>

      <!-- Вкладка: Печать -->
      <div v-if="currentTab === 'print'" class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        
        <!-- Форма добавления в очередь -->
        <div class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
          <h2 class="text-xl font-bold mb-4">Новый путевой лист</h2>
          <div class="space-y-3">
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Дата</label>
                <input type="text" v-model="selectedDate" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="ДД.ММ.ГГГГ">
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Водитель</label>
                <select v-model="selectedDriver" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
                  <option value="" disabled>Выберите...</option>
                  <option v-for="d in drivers" :key="d.id" :value="d.id">{{ d.name }}</option>
                </select>
              </div>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Техника</label>
              <select v-model="selectedVehicle" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
                <option value="" disabled>Выберите технику...</option>
                <option v-for="v in vehicles" :key="v.id" :value="v.id">{{ v.name }} {{ v.license_plate ? '('+v.license_plate+')' : '' }}</option>
              </select>
            </div>

            <!-- Режим трактора -->
            <div v-if="isTractorSelected" class="border-t border-gray-200 pt-3">
              <div class="flex items-center gap-4 mb-3">
                <label class="flex items-center gap-2 text-sm">
                  <input type="radio" value="cargo" v-model="tractorMode" class="accent-brand">
                  <span>Перевозка груза</span>
                </label>
                <label class="flex items-center gap-2 text-sm">
                  <input type="radio" value="field" v-model="tractorMode" class="accent-brand">
                  <span>Полевые работы</span>
                </label>
              </div>
            </div>

            <div class="border-t border-gray-200 pt-3">
              <div class="flex justify-between items-center mb-2">
                <span class="font-semibold text-sm text-gray-700">
                  {{ isTractorSelected && tractorMode === 'field' ? 'Задание на полевые работы' : 'Задание водителю' }}
                </span>
                <button v-if="(isTractorSelected && tractorMode === 'field' ? fieldRows : taskRows).length < 3" @click="isTractorSelected && tractorMode === 'field' ? addFieldRow() : addTaskRow()" type="button" class="text-xs bg-gray-100 hover:bg-gray-200 px-2 py-1 rounded border border-gray-300">+ Добавить строку</button>
              </div>

              <!-- Перевозочные строки -->
              <template v-if="!isTractorSelected || tractorMode === 'cargo'">
                <div v-for="(row, idx) in taskRows" :key="idx" class="bg-gray-50 p-3 rounded-lg border border-gray-200 mb-2 relative">
                  <div class="absolute top-2 right-2 text-xs text-gray-400">#{{ idx + 1 }}</div>
                  <div v-if="taskRows.length > 1" class="absolute top-2 right-8">
                    <button @click="removeTaskRow(idx)" type="button" class="text-red-500 hover:text-red-700 text-xs">&times;</button>
                  </div>
                  <div class="grid grid-cols-2 gap-2 mb-2">
                    <input type="text" v-model="row.customer" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="В чье распоряжение">
                    <input type="text" v-model="row.cargo" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Наименование груза">
                  </div>
                  <div class="grid grid-cols-2 gap-2 mb-2">
                    <input type="text" v-model="row.loading_point" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Пункт погрузки">
                    <input type="text" v-model="row.unloading_point" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Пункт разгрузки">
                  </div>
                  <div class="grid grid-cols-4 gap-2">
                    <input type="text" v-model="row.arrival_time" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Время приб.">
                    <input type="text" v-model="row.trips" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Ездок">
                    <input type="text" v-model="row.distance" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Расст., км">
                    <input type="text" v-model="row.tons" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Тонн">
                  </div>
                </div>
              </template>

              <!-- Полевые строки -->
              <template v-else>
                <div v-for="(row, idx) in fieldRows" :key="idx" class="bg-gray-50 p-3 rounded-lg border border-gray-200 mb-2 relative">
                  <div class="absolute top-2 right-2 text-xs text-gray-400">#{{ idx + 1 }}</div>
                  <div v-if="fieldRows.length > 1" class="absolute top-2 right-8">
                    <button @click="removeFieldRow(idx)" type="button" class="text-red-500 hover:text-red-700 text-xs">&times;</button>
                  </div>
                  <div class="grid grid-cols-3 gap-2 mb-2">
                    <input type="text" v-model="row.object" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Объект (поле, участок)">
                    <input type="text" v-model="row.area" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Площадь, га">
                    <input type="text" v-model="row.motohours" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Моточасы">
                  </div>
                  <div class="grid grid-cols-3 gap-2">
                    <input type="text" v-model="row.norm" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Норма выработки">
                    <input type="text" v-model="row.fact" class="w-full px-2 py-1.5 border border-gray-300 rounded text-sm" placeholder="Факт">
                  </div>
                </div>
              </template>
            </div>

            <button @click="addToQueue" class="w-full bg-gray-100 hover:bg-gray-200 text-gray-800 font-semibold py-2 px-4 rounded-lg shadow-sm border border-gray-300 transition">
              + Добавить в список
            </button>
          </div>
        </div>

        <!-- Очередь -->
        <div class="bg-white p-6 rounded-xl shadow-md border border-gray-200 flex flex-col">
          <h2 class="text-xl font-bold mb-4">Список на печать ({{ printQueue.length }})</h2>
          <div class="flex-1 overflow-y-auto mb-4 border border-gray-200 rounded-lg bg-gray-50 p-2 min-h-[200px]">
            <div v-if="printQueue.length === 0" class="text-gray-400 text-center mt-10">Список пуст</div>
            <div v-for="item in printQueue" :key="item.id" class="bg-white p-3 rounded shadow-sm border border-gray-200 mb-2 flex justify-between items-center">
              <div class="text-sm">
                <div class="font-bold">{{ item.driver_name }}</div>
                <div class="text-gray-600">{{ item.vehicle_name }}</div>
                <div class="text-xs text-gray-500">
                  <span v-for="(t, i) in item.tasks" :key="i">
                    {{ t.loading_point || '—' }}<span v-if="t.unloading_point"> &rarr; {{ t.unloading_point }}</span><span v-if="t.customer"> ({{ t.customer }})</span><span v-if="i < item.tasks.length - 1">; </span>
                  </span>
                </div>
                <div class="text-xs text-gray-400">{{ item.date }}</div>
              </div>
              <button @click="removeFromQueue(item.id)" class="text-red-500 hover:text-red-700 p-2 text-xl">&times;</button>
            </div>
          </div>
          
          <button @click="printBatch" class="w-full bg-brand hover:bg-blue-600 text-white font-semibold py-3 px-4 rounded-lg shadow transition active:scale-[0.98]">
            🖨️ Сгенерировать PDF
          </button>
          <div v-if="statusMessage" class="text-center text-sm font-medium mt-2" :class="statusMessage.includes('Ошибка') ? 'text-red-500' : 'text-green-600'">
            {{ statusMessage }}
          </div>
        </div>
      </div>

      <!-- Вкладка: Реквизиты -->
      <div v-if="currentTab === 'settings'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200 max-w-3xl mx-auto">
        <h2 class="text-xl font-bold mb-4">Реквизиты организации</h2>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Название организации</label>
            <input type="text" v-model="companySettings.company_name" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Адрес / телефон</label>
            <input type="text" v-model="companySettings.company_address" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">ИНН</label>
            <input type="text" v-model="companySettings.company_inn" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
          </div>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Диспетчер</label>
              <input type="text" v-model="companySettings.dispatcher_name" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Механик</label>
              <input type="text" v-model="companySettings.mechanic_name" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Медик</label>
              <input type="text" v-model="companySettings.medic_name" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
            </div>
          </div>
          <button @click="saveSettings" class="w-full bg-brand hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded-lg shadow transition active:scale-[0.98]">
            Сохранить реквизиты
          </button>
        </div>
      </div>

      <!-- Вкладка: Значения по умолчанию -->
      <div v-if="currentTab === 'defaults'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200 max-w-3xl mx-auto">
        <h2 class="text-xl font-bold mb-4">Значения по умолчанию</h2>
        <div class="space-y-4">
          <div class="font-semibold text-sm text-gray-700 border-b border-gray-200 pb-1">Перевозка / общие</div>
          <div class="grid grid-cols-2 gap-3">
            <input type="text" v-model="defaultValues.customer" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="В чье распоряжение">
            <input type="text" v-model="defaultValues.cargo" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Наименование груза">
          </div>
          <div class="grid grid-cols-2 gap-3">
            <input type="text" v-model="defaultValues.loading_point" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Пункт погрузки">
            <input type="text" v-model="defaultValues.unloading_point" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Пункт разгрузки">
          </div>
          <div class="grid grid-cols-4 gap-3">
            <input type="text" v-model="defaultValues.arrival_time" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Время приб.">
            <input type="text" v-model="defaultValues.trips" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Ездок">
            <input type="text" v-model="defaultValues.distance" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Расст., км">
            <input type="text" v-model="defaultValues.tons" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Тонн">
          </div>

          <div class="font-semibold text-sm text-gray-700 border-b border-gray-200 pb-1 pt-2">Полевые работы</div>
          <div class="grid grid-cols-3 gap-3">
            <input type="text" v-model="defaultValues.field_object" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Объект (поле, участок)">
            <input type="text" v-model="defaultValues.field_area" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Площадь, га">
            <input type="text" v-model="defaultValues.field_motohours" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Моточасы">
          </div>
          <div class="grid grid-cols-3 gap-3">
            <input type="text" v-model="defaultValues.field_norm" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Норма выработки">
            <input type="text" v-model="defaultValues.field_fact" class="w-full px-3 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="Факт">
          </div>

          <button @click="saveDefaults" class="w-full bg-brand hover:bg-blue-600 text-white font-semibold py-2 px-4 rounded-lg shadow transition active:scale-[0.98]">
            Сохранить значения по умолчанию
          </button>
        </div>
      </div>

      <!-- Вкладка: Водители -->
      <div v-if="currentTab === 'drivers'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Справочник водителей</h2>
        <form @submit.prevent="saveItem('driver', formDriver)" class="flex flex-wrap gap-2 mb-6 items-end">
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">ФИО</label><input required v-model="formDriver.name" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">В/У</label><input v-model="formDriver.driving_license" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">Удост. тракториста</label><input v-model="formDriver.tractor_license" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">СНИЛС</label><input v-model="formDriver.snils" class="w-full border p-2 rounded"></div>
          <button type="submit" class="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded font-bold">{{ formDriver.id ? 'Сохранить' : 'Добавить' }}</button>
          <button type="button" v-if="formDriver.id" @click="clearForm('driver')" class="bg-gray-300 px-4 py-2 rounded">Отмена</button>
        </form>
        <div class="overflow-x-auto">
          <table class="w-full text-left border-collapse text-sm">
            <thead><tr class="bg-gray-100 border-b"><th class="p-2">ФИО</th><th class="p-2">В/У</th><th class="p-2">Тракторные</th><th class="p-2">СНИЛС</th><th class="p-2 w-24">Действия</th></tr></thead>
            <tbody>
              <tr v-for="d in drivers" :key="d.id" class="border-b hover:bg-gray-50">
                <td class="p-2">{{ d.name }}</td><td class="p-2">{{ d.driving_license }}</td><td class="p-2">{{ d.tractor_license }}</td><td class="p-2">{{ d.snils }}</td>
                <td class="p-2">
                  <button @click="editDriver(d)" class="text-blue-500 mr-2">✎</button>
                  <button @click="deleteItem('driver', d.id)" class="text-red-500">🗑</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Вкладка: Техника -->
      <div v-if="currentTab === 'vehicles'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Справочник техники</h2>
        <form @submit.prevent="saveItem('vehicle', formVehicle)" class="flex flex-wrap gap-2 mb-6 items-end">
          <div class="flex-1 min-w-[150px]">
            <label class="block text-xs text-gray-500">Тип</label>
            <select v-model="formVehicle.vehicle_type" class="w-full border p-2 rounded">
              <option value="Грузовой">Грузовой</option>
              <option value="Трактор">Трактор</option>
            </select>
          </div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">Категория</label><input v-model="formVehicle.category" class="w-full border p-2 rounded" placeholder="например, B, C1, D"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">Марка/Модель</label><input required v-model="formVehicle.name" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[120px]"><label class="block text-xs text-gray-500">Госномер</label><input v-model="formVehicle.license_plate" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[120px]"><label class="block text-xs text-gray-500">СТС / ПСМ</label><input v-model="formVehicle.sts" class="w-full border p-2 rounded"></div>
          <button type="submit" class="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded font-bold">{{ formVehicle.id ? 'Сохранить' : 'Добавить' }}</button>
          <button type="button" v-if="formVehicle.id" @click="clearForm('vehicle')" class="bg-gray-300 px-4 py-2 rounded">Отмена</button>
        </form>
        <div class="overflow-x-auto">
          <table class="w-full text-left border-collapse text-sm">
            <thead><tr class="bg-gray-100 border-b"><th class="p-2">Тип</th><th class="p-2">Категория</th><th class="p-2">Модель</th><th class="p-2">Госномер</th><th class="p-2">СТС / ПСМ</th><th class="p-2 w-24">Действия</th></tr></thead>
            <tbody>
              <tr v-for="v in vehicles" :key="v.id" class="border-b hover:bg-gray-50">
                <td class="p-2">{{ v.vehicle_type }}</td><td class="p-2">{{ v.category }}</td><td class="p-2">{{ v.name }}</td><td class="p-2">{{ v.license_plate }}</td><td class="p-2">{{ v.sts }}</td>
                <td class="p-2">
                  <button @click="editVehicle(v)" class="text-blue-500 mr-2">✎</button>
                  <button @click="deleteItem('vehicle', v.id)" class="text-red-500">🗑</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Вкладка: Виды работ -->
      <div v-if="currentTab === 'works'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Виды работ</h2>
        <form @submit.prevent="saveItem('work', formWork)" class="flex flex-wrap gap-2 mb-6 items-end">
          <div class="flex-1 min-w-[200px]"><label class="block text-xs text-gray-500">Наименование</label><input required v-model="formWork.name" class="w-full border p-2 rounded"></div>
          <button type="submit" class="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded font-bold">{{ formWork.id ? 'Сохранить' : 'Добавить' }}</button>
          <button type="button" v-if="formWork.id" @click="clearForm('work')" class="bg-gray-300 px-4 py-2 rounded">Отмена</button>
        </form>
        <div class="overflow-x-auto">
          <table class="w-full text-left border-collapse text-sm">
            <thead><tr class="bg-gray-100 border-b"><th class="p-2">Вид работы</th><th class="p-2 w-24">Действия</th></tr></thead>
            <tbody>
              <tr v-for="w in works" :key="w.id" class="border-b hover:bg-gray-50">
                <td class="p-2">{{ w.name }}</td>
                <td class="p-2">
                  <button @click="editWork(w)" class="text-blue-500 mr-2">✎</button>
                  <button @click="deleteItem('work', w.id)" class="text-red-500">🗑</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

    </div>
  `
};

createApp(App).mount('#app');