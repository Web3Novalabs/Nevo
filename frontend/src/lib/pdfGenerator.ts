import { jsPDF } from "jspdf";

export interface DonationDetails {
  poolTitle: string;
  amount: string;
  asset: string;
  date: Date;
  receiptId: string;
}

export const generateTaxReceiptPDF = (details: DonationDetails) => {
  const doc = new jsPDF();

  // Colors and styling
  const primaryColor = "#0f172a"; // slate-900
  const secondaryColor = "#64748b"; // slate-500
  const accentColor = "#3b82f6"; // blue-500

  // Header Title
  doc.setFont("helvetica", "bold");
  doc.setFontSize(22);
  doc.setTextColor(primaryColor);
  doc.text("Official Tax Receipt", 20, 30);

  // Line separator
  doc.setDrawColor(203, 213, 225); // slate-300
  doc.line(20, 35, 190, 35);

  // Receipt Details (Top Right)
  doc.setFont("helvetica", "normal");
  doc.setFontSize(10);
  doc.setTextColor(secondaryColor);
  doc.text(`Receipt ID: ${details.receiptId}`, 190, 20, { align: "right" });
  doc.text(`Date: ${details.date.toLocaleDateString()}`, 190, 26, { align: "right" });

  // Main Content
  doc.setFontSize(12);
  doc.setTextColor(primaryColor);
  
  let yPos = 55;
  
  doc.setFont("helvetica", "bold");
  doc.text("Thank you for your generous donation!", 20, yPos);
  
  yPos += 15;
  doc.setFont("helvetica", "normal");
  doc.text("This receipt acknowledges that you have made a charitable contribution to the", 20, yPos);
  yPos += 7;
  doc.text("following campaign/pool:", 20, yPos);

  yPos += 12;
  doc.setFont("helvetica", "bold");
  doc.text(`${details.poolTitle}`, 25, yPos);

  yPos += 20;

  // Donation Box
  doc.setFillColor(248, 250, 252); // slate-50
  doc.roundedRect(20, yPos, 170, 30, 3, 3, "F");
  doc.setDrawColor(226, 232, 240); // slate-200
  doc.roundedRect(20, yPos, 170, 30, 3, 3, "D");

  doc.setFont("helvetica", "normal");
  doc.setTextColor(secondaryColor);
  doc.text("Donation Amount:", 30, yPos + 18);

  doc.setFont("helvetica", "bold");
  doc.setFontSize(16);
  doc.setTextColor(accentColor);
  doc.text(`${details.amount} ${details.asset}`, 80, yPos + 18);

  yPos += 50;

  // Disclaimer / Footer
  doc.setFont("helvetica", "italic");
  doc.setFontSize(10);
  doc.setTextColor(secondaryColor);
  const footerText = 
    "Please keep this receipt for your tax records. We confirm that no goods or\n" + 
    "services were provided in exchange for this contribution.";
  
  doc.text(footerText, 20, yPos);

  // Save the PDF
  doc.save(`Tax_Receipt_${details.receiptId}.pdf`);
};
