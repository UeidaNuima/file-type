function getBaseLog(x, y) {
  const res = Math.log(y) / Math.log(x);
  if (res - Math.floor(res) < 0.000000001) {
    return Math.floor(res);
  }
}

const block_width = 30;
let block_offset = 0;
// 三进制，每一位最多可以存储 3 个数字
let alpha = 3;
let n = 18;
let l_s = 9;
let originalStringSeries = [
  0, 0, 1, 0, 1, 0, 2, 1, 0, 2, 1, 0, 2, 1, 2, 0, 2, 1, 0, 2, 1, 2, 0, 0,
];

let l_c1 = Math.ceil(getBaseLog(alpha, n - l_s));
let l_c2 = Math.ceil(getBaseLog(alpha, l_s));
let l_c3 = 1;

let indicators = [];
let cs = [];

/**
 *
 * @param {number} num
 * @param {number} base
 * @param {number} padding
 */
function formatToBaseAndPadding(num, base, padding) {
  return num.toString(base).padStart(padding, '0');
}

/**
 *
 * @param {Array<string>} stringSeries
 * @param {number | undefined} offset
 */
function renderStringSeries(stringSeries) {
  let container = document.getElementById('original-str');
  for (let i = 0; i < n - l_s; i++) {
    stringSeries.unshift(0);
  }
  for (let item of stringSeries) {
    let itemNode = document.createElement('div');
    itemNode.classList.add('original-str-item');
    itemNode.textContent = item;
    container.appendChild(itemNode);
  }
}

function initSlidingWindow() {
  let container = document.getElementById('sliding-window');
  for (let i = 0; i < n; i++) {
    let itemNode = document.createElement('div');
    itemNode.classList.add('sliding-window-item');
    if (i < n - l_s) {
      itemNode.classList.add('sliding-window-item-forward-search');
    } else {
      itemNode.classList.add('sliding-window-item-pending-encoding');
    }
    container.appendChild(itemNode);
  }
}

function moveWindow(i) {
  block_offset += i;
  const div = document.getElementById('original-str');
  div.style.left = `${-block_offset * block_width}px`;
  if (originalStringSeries.length - block_offset <= n - l_s) {
    let runBtn = document.getElementById('run');
    runBtn.disabled = true;
    runBtn.title = 'No items left in window';
  }
}

function randomBetweenMinus1And1() {
  return Math.random() * 2 - 1;
}

function createIndicator(left, right) {
  let indicator = document.createElement('div');
  indicator.className = 'block-indicator';
  indicator.style.height = `${
    Math.ceil(block_width / 2) + randomBetweenMinus1And1() * 10
  }px`;
  indicator.style.width = `${(right - left - 1) * block_width}px`;
  // indicator.style.background = 'red';
  indicator.style.left = `${(left - 0.5) * block_width}px`;
  // indicator.style.position = `absolute`;
  indicators.push(indicator);
  let container = document.getElementById('indicator-container');
  container.appendChild(indicator);
}

function clearIndicators() {
  let container = document.getElementById('indicator-container');
  for (let indicator of indicators) {
    container.removeChild(indicator);
  }
  indicators = [];
}

/**
 *
 * @param {string[]} stringSeries
 */
function detectReproducibleExtension(stringSeries) {
  let j = n - l_s;
  let p = 0;
  let maxL = 0;
  let s = 0;
  for (let i = 1; i <= j; i++) {
    for (let l = 1; l <= n - 1 - j; l++) {
      if (stringSeries[i + (l - 1) - 1] != stringSeries[j + 1 + (l - 1) - 1]) {
        break;
      }
      if (l >= maxL) {
        p = i;
        maxL = l;
        s = stringSeries[j + 1 + (l - 1) - 1 + 1];
      }
    }
  }
  return {
    l: maxL + 1,
    p,
    s,
  };
}

async function renderCs() {
  let container = document.getElementById('cs');
  container.innerHTML = '';
  for (let c of cs) {
    let node = document.createElement('div');
    node.innerHTML = `p=${c.p}; l=${c.l}; s=${c.s}; C=${c.c}`;
    container.appendChild(node);
  }
}

function nextRoundStep1() {
  const { l, p, s } = detectReproducibleExtension(
    originalStringSeries.slice(0 + block_offset, n + block_offset)
  );

  cs.push({
    p,
    l,
    s,
    c: `${formatToBaseAndPadding(p - 1, alpha, l_c1)}${formatToBaseAndPadding(
      l - 1,
      alpha,
      l_c2
    )}${s}`,
  });

  renderCs();

  clearIndicators();
  createIndicator(p, p + l - 1);
  createIndicator(n - l_s + 1, n - l_s + l - 1 + 1);
  return { l, p, s };
}

function nextRoundStep2(l) {
  clearIndicators();
  moveWindow(l);
}

let currentRound = 1;
let currentL = 0;

function roundLoop() {
  if (currentRound === 1) {
    const { l } = nextRoundStep1();
    currentL = l;
    currentRound = 2;
  } else {
    nextRoundStep2(currentL);
    currentRound = 1;
  }
}

window.onload = function () {
  renderStringSeries(originalStringSeries);
  initSlidingWindow();
};
