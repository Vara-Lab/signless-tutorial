import { useAccount, useProgram, usePrepareProgramTransaction } from '@gear-js/react-hooks';
import {
  useSignlessTransactions,
  EzTransactionsSwitch,
  usePrepareEzTransactionParams,
  useGaslessTransactions,
} from 'gear-ez-transactions';
import { useState, useEffect, useRef } from 'react';
import { useSignAndSend } from '@/hooks/use-sign-and-send';
import { Program } from '@/hocs/lib';

const ALLOWED_SIGNLESS_ACTIONS = ['SayHello', 'SayPersonalHello'];

export function EzSwitchAndSendHello() {
  const { account } = useAccount();
  const signless = useSignlessTransactions();
  const gasless = useGaslessTransactions();

  const { data: program } = useProgram({
    library: Program,
    id: import.meta.env.VITE_PROGRAMID,
  });

  const { prepareTransactionAsync } = usePrepareProgramTransaction({
    program,
    serviceName: 'service',
    functionName: 'helloWorld',
  });

  const { prepareEzTransactionParams } = usePrepareEzTransactionParams();
  const { signAndSend } = useSignAndSend();

  const [loading, setLoading] = useState(false);
  const [voucherPending, setVoucherPending] = useState(false);

  const hasRequestedOnceRef = useRef(false);

  // Reset voucher request flag when account changes
  useEffect(() => {
    hasRequestedOnceRef.current = false;
  }, [account?.address]);

  // ─────────── Controlled voucher request ───────────
  useEffect(() => {
    if (!account?.address || !gasless.isEnabled || hasRequestedOnceRef.current) return;

    hasRequestedOnceRef.current = true;
    setVoucherPending(true);

    const requestVoucherSafely = async () => {
      try {
        const alreadyEnabled = gasless.voucherStatus?.enabled;
        if (alreadyEnabled) {
          console.log('ℹ️ Voucher already active, skipping request.');
          setVoucherPending(false);
          return;
        }

        console.log('⚙️ Requesting voucher...');
        await gasless.requestVoucher(account.address);

        let retries = 5;
        while (retries-- > 0) {
          await new Promise((res) => setTimeout(res, 300)); 
          if (gasless.voucherStatus?.enabled) {
            console.log('✅ Voucher activation confirmed');
            setVoucherPending(false);
            return;
          }
        }

        console.warn('⚠️ Voucher was not activated after the request.');
        setVoucherPending(false);
      } catch (err) {
        console.error('❌ Error requesting voucher:', err);
        hasRequestedOnceRef.current = false; 
        setVoucherPending(false);
      }
    };

    void requestVoucherSafely(); 
  }, [account?.address, gasless.isEnabled]);

  // ─────────── Send HelloWorld transaction ───────────
  const handleSendHello = async () => {
    if (!signless.isActive) {
      alert('Signless session is not active');
      return;
    }

    setLoading(true);
    try {
      const { sessionForAccount, ...params } = await prepareEzTransactionParams(false);
      if (!sessionForAccount) throw new Error('Missing sessionForAccount');

      const { transaction } = await prepareTransactionAsync({
        args: [null],
        value: 0n,
        ...params,
      });

      signAndSend(transaction, {
        onSuccess: () => setLoading(false),
        onError: () => setLoading(false),
      });
    } catch (e) {
      console.error('prepare/send failed:', e);
      setLoading(false);
    }
  };

  const voucherEnabled = gasless.voucherStatus?.enabled;
  const signlessActive = signless.isActive;

  return (
    <div style={styles.container}>
      <p style={styles.description}>
        {voucherEnabled
          ? '✅ Gasless session active.'
          : gasless.isEnabled
          ? voucherPending
            ? '⏳ Requesting voucher…'
            : '🛠️ Waiting for voucher activation…'
          : '⚠️ Gasless service unavailable.'}
      </p>

      <EzTransactionsSwitch allowedActions={ALLOWED_SIGNLESS_ACTIONS} />

      <p style={{ color: signlessActive ? 'green' : 'red' }}>
        {signlessActive ? '✅ Signless session active' : '⚠️ Signless session inactive'}
      </p>

      <button
        style={{ ...styles.button, opacity: loading || !signlessActive ? 0.6 : 1 }}
        onClick={handleSendHello}
        disabled={loading || !signlessActive}
      >
        {loading ? 'Sending…' : 'Send HelloWorld'}
      </button>
    </div>
  );
}

// ─────────── Styles ───────────
const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '1.5rem',
    background: '#f0fdf4',
    borderRadius: '1rem',
    boxShadow: '0 6px 18px rgba(0,0,0,0.05)',
    textAlign: 'center',
    maxWidth: '420px',
    margin: '2rem auto',
    fontFamily: 'Inter, sans-serif',
  },
  description: {
    marginBottom: '0.75rem',
    color: '#4b5563',
  },
  button: {
    marginTop: '1rem',
    background: '#6366f1',
    color: '#fff',
    fontWeight: 'bold',
    fontSize: '1rem',
    padding: '0.75rem 1.25rem',
    borderRadius: '0.75rem',
    border: 'none',
    cursor: 'pointer',
    transition: 'background 0.3s',
    width: '100%',
  },
};
